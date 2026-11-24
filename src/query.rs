//! PHP parse_str / http_build_query semantics, not WHATWG URL normalization.

#[derive(Debug)]
enum Value {
    Scalar(Vec<u8>),
    Array(Parameters),
}

#[derive(Debug, Default)]
pub(crate) struct Parameters(Vec<(Vec<u8>, Value)>);

impl Parameters {
    fn insert(&mut self, key: Vec<u8>, value: Value) {
        if let Some((_, previous)) = self.0.iter_mut().find(|(name, _)| *name == key) {
            *previous = value;
        } else {
            self.0.push((key, value));
        }
    }

    pub(crate) fn scalar(&self, key: &str) -> Option<&[u8]> {
        match &self.0.iter().find(|(name, _)| name == key.as_bytes())?.1 {
            Value::Scalar(value) => Some(value),
            Value::Array(_) => None,
        }
    }

    fn assign(&mut self, path: &[Option<Vec<u8>>], value: Vec<u8>) {
        let key = match &path[0] {
            Some(key) => key.clone(),
            None => {
                let next = self
                    .0
                    .iter()
                    .filter_map(|(key, _)| numeric_key(key))
                    .max()
                    .map_or(Some(0), |max| max.checked_add(1));
                let Some(next) = next else {
                    return;
                };
                next.to_string().into_bytes()
            }
        };
        if path.len() == 1 {
            self.insert(key, Value::Scalar(value));
            return;
        }
        let index = match self.0.iter().position(|(name, _)| *name == key) {
            Some(index) => index,
            None => {
                self.0.push((key, Value::Array(Self::default())));
                self.0.len() - 1
            }
        };
        let entry = &mut self.0[index].1;
        if !matches!(entry, Value::Array(_)) {
            *entry = Value::Array(Self::default());
        }
        if let Value::Array(array) = entry {
            array.assign(&path[1..], value);
        }
    }

    fn encode(&self, prefix: Option<&[u8]>, output: &mut Vec<String>) {
        for (name, value) in &self.0 {
            let key = if let Some(prefix) = prefix {
                [prefix, b"[", name, b"]"].concat()
            } else {
                name.clone()
            };
            match value {
                Value::Scalar(value) => output.push(format!("{}={}", encode(&key), encode(value))),
                Value::Array(array) => array.encode(Some(&key), output),
            }
        }
    }

    fn query(&self) -> String {
        let mut output = Vec::new();
        self.encode(None, &mut output);
        output.join("&")
    }
}

fn numeric_key(key: &[u8]) -> Option<i64> {
    let text = std::str::from_utf8(key).ok()?;
    let number: i64 = text.parse().ok()?;
    (number.to_string() == text).then_some(number)
}

fn encode(bytes: &[u8]) -> String {
    // PHP uses RFC 1738: unlike HTML form encoding, '*' must also be escaped.
    url::form_urlencoded::byte_serialize(bytes)
        .collect::<String>()
        .replace('*', "%2A")
}

fn decode(text: &str) -> Vec<u8> {
    percent_encoding::percent_decode_str(&text.replace('+', " ")).collect()
}

pub(crate) fn parameters(url: &str) -> Parameters {
    let mut parameters = Parameters::default();
    let query = url.split_once('?').map_or("", |(_, query)| query);
    let query = query.split('\0').next().unwrap_or_default();
    // Match PHP's default max_input_vars and max_input_nesting_level.
    for pair in query.split('&').filter(|pair| !pair.is_empty()).take(1000) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let key = decode(key);
        let key = key.split(|byte| *byte == 0).next().unwrap_or_default();
        let key = &key[key.iter().take_while(|byte| **byte == b' ').count()..];
        if key.is_empty() {
            continue;
        }
        let bracket = key.iter().position(|byte| *byte == b'[');
        let root = &key[..bracket.unwrap_or(key.len())];
        if root.is_empty() {
            continue;
        }
        let root: Vec<u8> = root
            .iter()
            .map(|byte| {
                if matches!(byte, b' ' | b'.') {
                    b'_'
                } else {
                    *byte
                }
            })
            .collect();
        let mut path = vec![Some(root.clone())];
        let mut rest = &key[bracket.unwrap_or(key.len())..];
        while rest.starts_with(b"[") {
            if path.len() > 64 {
                path.push(None);
                break;
            }
            let Some(end) = rest.iter().position(|byte| *byte == b']') else {
                if path.len() == 1 {
                    path[0] = Some(
                        key.iter()
                            .map(|byte| {
                                if matches!(byte, b' ' | b'.' | b'[') {
                                    b'_'
                                } else {
                                    *byte
                                }
                            })
                            .collect(),
                    );
                }
                break;
            };
            let index = &rest[1..end];
            path.push(
                if index.is_empty()
                    || (index.len() == 1 && matches!(index[0], b' ' | b'\t'..=b'\r'))
                {
                    None
                } else {
                    Some(index.to_vec())
                },
            );
            rest = &rest[end + 1..];
        }
        if path.len() > 65 {
            parameters.0.retain(|(name, _)| *name != root);
            continue;
        }
        parameters.assign(&path, decode(value));
    }
    parameters
}

pub(crate) fn without_parameters(url: &str, names: &[&str]) -> String {
    let mut parameters = parameters(url);
    parameters
        .0
        .retain(|(name, _)| !names.iter().any(|unwanted| name == unwanted.as_bytes()));
    let base = url.split_once('?').map_or(url, |(base, _)| base);
    if parameters.0.is_empty() {
        base.to_owned()
    } else {
        format!("{base}?{}", parameters.query())
    }
}

pub(crate) fn add_parameters(url: &str, additions: &[(&str, &str)]) -> String {
    let mut parameters = parameters(url);
    for (name, value) in additions {
        parameters.insert(
            name.as_bytes().to_vec(),
            Value::Scalar(value.as_bytes().to_vec()),
        );
    }
    // PHP's array_merge renumbers integer keys at the top level only.
    let mut next = 0;
    for (name, _) in &mut parameters.0 {
        if numeric_key(name).is_some() {
            *name = next.to_string().into_bytes();
            next += 1;
        }
    }
    let base = url.split_once('?').map_or(url, |(base, _)| base);
    format!("{base}?{}", parameters.query())
}

pub(crate) fn integer(text: &str) -> Option<i64> {
    let text = text.trim_matches([' ', '\t', '\r', '\n', '\x0b', '\x0c']);
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
        || (digits.len() > 1 && digits.starts_with('0'))
    {
        return None;
    }
    text.parse().ok()
}
