<?php

// Run the actual pinned PHP package, never a reimplementation of its algorithm.
// PHP_URL_SIGNER_SOURCE points to a checkout at the revision in docs/usage.md.
$source = getenv('PHP_URL_SIGNER_SOURCE');
if (! $source || ! is_file($source.'/src/Sha256UrlSigner.php')) {
    fwrite(STDERR, "Set PHP_URL_SIGNER_SOURCE to the reference PHP checkout\n");
    exit(1);
}
spl_autoload_register(function ($class) use ($source) {
    $prefix = 'Spatie\\UrlSigner\\';
    if (str_starts_with($class, $prefix)) {
        require $source.'/src/'.str_replace('\\', '/', substr($class, strlen($prefix))).'.php';
    }
});

trait FrozenClock
{
    public int $now;

    protected function isFuture(int $timestamp): bool
    {
        // Translate the frozen clock, but exercise the upstream comparison itself.
        do {
            $before = time();
            $result = parent::isFuture($timestamp - $this->now + $before);
        } while (time() !== $before);

        return $result;
    }
}

class FrozenSha256 extends Spatie\UrlSigner\Sha256UrlSigner
{
    use FrozenClock;
}

class FrozenMd5 extends Spatie\UrlSigner\Md5UrlSigner
{
    use FrozenClock;
}

// Limit cases intentionally provoke these two warnings. Other warnings stay visible.
set_error_handler(function ($severity, $message) {
    return $severity === E_WARNING && (
        str_starts_with($message, 'parse_str(): Input variables exceeded') ||
        str_starts_with($message, 'parse_str(): Input variable nesting level exceeded')
    );
});

$results = [];
foreach (json_decode(stream_get_contents(STDIN), true, flags: JSON_THROW_ON_ERROR) as $case) {
    $class = $case['md5'] ? FrozenMd5::class : FrozenSha256::class;
    $signer = new $class($case['key'], $case['expires_name'], $case['signature_name']);
    $signer->now = $case['now'];
    $signed = $signer->sign($case['url'], new DateTime('@'.$case['expiration']), $case['override_key']);
    $validation = [];
    foreach ([$case['expiration'] - 1, $case['expiration'], $case['expiration'] + 1] as $now) {
        $signer->now = $now;
        foreach (array_merge([$signed, $case['rust_signed']], $case['checks']) as $url) {
            $validation[] = $signer->validate($url, $case['override_key']);
        }
    }
    $results[] = ['signed' => $signed, 'validation' => $validation];
}
echo json_encode($results, JSON_THROW_ON_ERROR | JSON_UNESCAPED_SLASHES | JSON_PRETTY_PRINT), "\n";
