#!/usr/bin/env nu
use std/assert

use ../scripts/pw.nu
use ../scripts/pp.nu *

def open_page [html: string] {
    let b64 = ($html | encode base64 | into string | str replace -a "\n" "")
    let url = $"data:text/html;base64,($b64)"
    pw nav $url | ignore
    let ready = (pw wait-for "#prompt-textarea")
    if not $ready {
        error make { msg: "#prompt-textarea did not appear" }
    }
    sleep 150ms
}

def png_fixture []: nothing -> record {
    let dir = (mktemp -d)
    let b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO2O7NwAAAAASUVORK5CYII="
    let path = $"($dir)/pixel.png"
    $b64 | decode base64 | save -f $path

    {
        dir: $dir
        path: $path
        base64: $b64
    }
}

def "test debug-attachment-payload preserves png bytes and mime" [] {
    let fixture = (png_fixture)
    let payload = (pp debug-attachment-payload $fixture.path)
    let item = ($payload | first)

    assert equal "pixel.png" $item.name
    assert equal "image/png" $item.mime
    assert equal 68 $item.size
    assert equal $fixture.base64 $item.base64
}

def "test debug-attachment-payload pipeline input defaults to text" [] {
    let text = "hello\nworld"
    let expected_b64 = ($text | into binary | encode base64 | into string | str replace -a "\n" "")

    let payload = ($text | pp debug-attachment-payload --name "note.txt")
    let item = ($payload | first)

    assert equal "note.txt" $item.name
    assert equal "text/plain" $item.mime
    assert equal 11 $item.size
    assert equal $expected_b64 $item.base64
}

def "test debug-attach returns browser attachment metadata" [] {
    open_page "<div id='prompt-textarea' contenteditable='true'></div>"
    let fixture = (png_fixture)

    let result = (pp debug-attach $fixture.path)
    let attached = ($result.attachments | first)

    assert equal true $result.attached
    assert equal "pixel.png" $attached.name
    assert equal "image/png" $attached.type
    assert equal 68 $attached.size
}

def main [] {
    def run-test [name: string, block: closure] {
        print -n $"Running ($name)... "
        try {
            do $block
            print "✓"
            { name: $name, ok: true }
        } catch {|e|
            print $"✗ ($e.msg)"
            { name: $name, ok: false, error: $e.msg }
        }
    }

    let results = [
        (run-test "test debug-attachment-payload preserves png bytes and mime" { test debug-attachment-payload preserves png bytes and mime })
        (run-test "test debug-attachment-payload pipeline input defaults to text" { test debug-attachment-payload pipeline input defaults to text })
        (run-test "test debug-attach returns browser attachment metadata" { test debug-attach returns browser attachment metadata })
    ]

    let passed = ($results | where ok == true | length)
    let failed = ($results | where ok == false | length)

    print $"\n($passed) passed, ($failed) failed"
    if $failed > 0 { exit 1 }
}
