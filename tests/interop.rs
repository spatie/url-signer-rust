//! Byte-for-byte PHP signatures and two-way validation, including expiry boundaries.

use std::{
    io::Write,
    process::{Command, Stdio},
};

use serde::{Deserialize, Serialize};
use spatie_url_signer::{Expiration, Md5UrlSigner, Sha256UrlSigner, SignatureAlgorithm, UrlSigner};

#[derive(Debug, Deserialize, Serialize)]
struct Case {
    url: String,
    key: String,
    override_key: Option<String>,
    expires_name: String,
    signature_name: String,
    md5: bool,
    now: i64,
    expiration: i64,
    rust_signed: String,
    checks: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Expected {
    signed: String,
    validation: Vec<bool>,
}

fn evaluate<A: SignatureAlgorithm + Default + Clone>(case: &Case) -> Expected {
    let signer = UrlSigner::<A>::new(&case.key)
        .unwrap()
        .with_parameter_names(&case.expires_name, &case.signature_name);
    let now = case.now;
    let signing = signer.clone().with_clock(move || now);
    let signed = signing
        .sign_with_key(
            &case.url,
            Expiration::Timestamp(case.expiration),
            case.override_key.as_deref().unwrap_or(&case.key),
        )
        .unwrap();
    let mut validation = Vec::new();
    for now in [case.expiration - 1, case.expiration, case.expiration + 1] {
        let verifying = signer.clone().with_clock(move || now);
        for url in [&signed, &case.rust_signed].into_iter().chain(&case.checks) {
            validation.push(
                verifying.validate_with_key(url, case.override_key.as_deref().unwrap_or(&case.key)),
            );
        }
    }
    Expected { signed, validation }
}

fn rust(case: &Case) -> Expected {
    if case.md5 {
        evaluate::<spatie_url_signer::Md5>(case)
    } else {
        evaluate::<spatie_url_signer::Sha256>(case)
    }
}

fn compare(cases: &[Case], expected: &[Expected]) {
    assert_eq!(cases.len(), expected.len());
    for (case, expected) in cases.iter().zip(expected) {
        let actual = rust(case);
        assert_eq!(actual.signed, expected.signed, "signing {case:?}");
        assert_eq!(
            actual.validation, expected.validation,
            "validating {case:?}"
        );
    }
}

#[test]
fn matches_committed_php_fixtures() {
    let (cases, expected): (Vec<Case>, Vec<Expected>) =
        serde_json::from_str(include_str!("php/fixtures.json")).unwrap();
    compare(&cases, &expected);
}

#[test]
#[ignore = "requires PHP and the pinned source checkout; executed by the PHP interoperability CI job"]
fn differential_against_php() {
    let mut cases = Vec::new();
    let queries = [
        "",
        "?",
        "?foo=bar&baz=qux",
        "?x=a%20b&y=a+b&z=%2B",
        "?x=~!*'()@:/?=é",
        "?x=first&y=middle&x=last",
        "?x[]=a&x[]=b",
        "?x[a][b]=c&x[a][d]=e",
        "?a.b=one&a+b=two",
        "?x=%FF%00%FE&%FF=bytes",
        "?x=%&y=%2G",
        "?flag&&empty=",
        "?expires=3&signature=old&foo=bar",
        "?signature[]=bad&expires[x]=bad",
        "#fragment",
        "?x=value#fragment",
        "#before?x=after",
        "?0=a&5=b&-1=c",
        "?x[-4]=a&x[]=b",
        "?x[9223372036854775807]=a&x[]=b",
        "?a[b]tail=c",
        "?a[=c",
        "?a[b][=c",
        "?a[ ]=a&a[  ]=b",
        "?%20%09a.b=x",
        "?%00a=b&a%00b=c",
        "?a=before\0&b=after",
        "?a[b][c][=x",
        "?a[\x0b]=x&a[\x0c]=y",
    ];
    for md5 in [false, true] {
        for (index, query) in queries.iter().enumerate() {
            for names in [("expires", "signature"), ("until", "hash"), ("0", "1")] {
                cases.push(make_case(
                    format!("HTTPS://MyApp.com:443/a/../b{query}"),
                    md5,
                    names,
                    index,
                ));
            }
        }
    }
    for count in [63, 64, 65] {
        for suffix in ["", "["] {
            cases.push(make_case(
                format!(
                    "https://myapp.com?a=old&a{}{}=value&b=retained",
                    "[b]".repeat(count),
                    suffix
                ),
                false,
                ("expires", "signature"),
                count,
            ));
        }
    }
    for count in [998, 999, 1000, 1001] {
        let query = (0..count)
            .map(|index| format!("q{index}=v"))
            .collect::<Vec<_>>()
            .join("&");
        cases.push(make_case(
            format!("https://myapp.com?{query}"),
            false,
            ("expires", "signature"),
            count,
        ));
    }
    // Seeded, reproducible combinations stress overwrites and nested array parsing.
    let atoms = [
        "a", "b", "a.b", "a+b", "a[]", "a[0]", "a[-3]", "a[02]", "a[b]", "a[b][]", "a[b][c]", "a[",
        "a[b][", "0", "17", "-1", "01", "%FF", "a%00b",
    ];
    let values = [
        "", "plain", "a+b", "%20%2B", "%FF%00", "#frag", "x=y", "~*.", "é",
    ];
    let mut seed = 1729_u64;
    for index in 0..2000 {
        let mut url = "relative/path?".to_owned();
        for pair in 0..8 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key = atoms[(seed >> 32) as usize % atoms.len()];
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = values[(seed >> 32) as usize % values.len()];
            if pair != 0 {
                url.push('&');
            }
            url.push_str(&format!("{key}={value}"));
        }
        cases.push(make_case(
            url,
            index % 2 == 0,
            ("expires", "signature"),
            index,
        ));
    }
    let mut child = Command::new("php")
        .args(["-d", "display_errors=stderr", "tests/php/oracle.php"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("PHP must be installed");
    let input = serde_json::to_vec(&cases).unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(&input).unwrap());
    let output = child.wait_with_output().unwrap();
    writer.join().unwrap();
    assert!(output.status.success());
    let expected: Vec<Expected> = serde_json::from_slice(&output.stdout).unwrap();
    compare(&cases, &expected);
    println!(
        "{} PHP/Rust cases; exact signed URLs and {} validation outcomes matched",
        cases.len(),
        expected
            .iter()
            .map(|result| result.validation.len())
            .sum::<usize>()
    );
}

fn make_case(url: String, md5: bool, names: (&str, &str), index: usize) -> Case {
    let mut case = Case {
        url,
        key: "random_monkey".into(),
        override_key: match index % 3 {
            0 => Some("custom-key".into()),
            1 => Some(String::new()),
            _ => None,
        },
        expires_name: names.0.into(),
        signature_name: names.1.into(),
        md5,
        now: 1_700_000_000,
        expiration: 4_000_000_000,
        rust_signed: String::new(),
        checks: Vec::new(),
    };
    case.rust_signed = if md5 {
        Md5UrlSigner::new(&case.key)
            .unwrap()
            .with_parameter_names(names.0, names.1)
            .with_clock(|| 1_700_000_000)
            .sign_with_key(
                &case.url,
                Expiration::Timestamp(case.expiration),
                case.override_key.as_deref().unwrap_or(&case.key),
            )
            .unwrap()
    } else {
        Sha256UrlSigner::new(&case.key)
            .unwrap()
            .with_parameter_names(names.0, names.1)
            .with_clock(|| 1_700_000_000)
            .sign_with_key(
                &case.url,
                Expiration::Timestamp(case.expiration),
                case.override_key.as_deref().unwrap_or(&case.key),
            )
            .unwrap()
    };
    for bad in [
        "",
        "abc",
        "4000000000tail",
        "4000000000.0",
        "1e10",
        "04000000000",
        "999999999999999999999999",
        "-1",
    ] {
        case.checks
            .push(format!("{}&{}={bad}", case.rust_signed, names.0));
    }
    for suffix in [
        format!("{}[]=4000000000", names.0),
        format!("{}[a][b]=4000000000", names.0),
        format!("{}[]=bad", names.1),
        format!("{}=bad", names.1),
        "tampered=true".into(),
    ] {
        case.checks.push(format!("{}&{suffix}", case.rust_signed));
    }
    case.checks.push(
        case.rust_signed
            .replace("relative/path", "relative/elsewhere"),
    );
    case.checks.extend([
        "https://myapp.com".into(),
        format!("https://myapp.com?{}=4000000000", names.0),
        format!("https://myapp.com?{}=bad", names.1),
    ]);
    for expiration in [
        "+4000000000",
        " 4000000000\t",
        "04000000000",
        "1e10",
        "4000000000.0",
        "9223372036854775808",
    ] {
        let key = case.override_key.as_deref().unwrap_or(&case.key);
        let signature = if md5 {
            spatie_url_signer::Md5.create_signature("https://myapp.com", expiration, key)
        } else {
            spatie_url_signer::Sha256.create_signature("https://myapp.com", expiration, key)
        };
        let encoded: String = url::form_urlencoded::byte_serialize(expiration.as_bytes()).collect();
        case.checks.push(format!(
            "https://myapp.com?{}={encoded}&{}={signature}",
            names.0, names.1
        ));
    }
    case
}
