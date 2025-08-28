use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Bencode Format for Announce String - Complete Demo ===\n");

    // Read the test torrent file
    let torrent_bytes = fs::read("test.torrent")?;

    println!("1. What is Bencode?");
    println!("   Bencode is a simple encoding format used by BitTorrent files.");
    println!("   It has 4 data types: strings, integers, lists, and dictionaries.\n");

    println!("2. Bencode Data Types:");
    println!("   Strings:      <length>:<string>     Example: '5:hello' = 'hello'");
    println!("   Integers:     i<integer>e           Example: 'i42e' = 42");
    println!("   Lists:        l<elements>e          Example: 'l5:helloi42ee' = ['hello', 42]");
    println!(
        "   Dictionaries: d<key><value>...e     Example: 'd3:key5:valuee' = {{'key': 'value'}}\n"
    );

    println!("3. Raw Torrent File (first 100 characters as ASCII):");
    let ascii_view: String = torrent_bytes
        .iter()
        .take(100)
        .map(|&b| {
            if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '?'
            }
        })
        .collect();
    println!("   {}\n", ascii_view);

    println!("4. Breaking Down the Announce Field:");
    println!("   The file starts: 'd8:announce22:udp://opentor.net:6969...'");
    println!("   ");
    println!("   'd'                        ← Dictionary start");
    println!("   '8:'                       ← Key length (8 bytes)");
    println!("   'announce'                 ← The key name");
    println!("   '22:'                      ← Value length (22 bytes)");
    println!("   'udp://opentor.net:6969'   ← The announce URL");
    println!("   '13:announce-list'         ← Next key (13 bytes: 'announce-list')");
    println!("   'l'                        ← List start (for announce-list value)");
    println!("   ...\n");

    println!("5. Hex View of First 50 Bytes:");
    print!("   ");
    for (i, byte) in torrent_bytes.iter().take(50).enumerate() {
        if i % 16 == 0 && i > 0 {
            println!();
            print!("   ");
        }
        print!("{:02x} ", byte);
    }
    println!("\n");

    println!("6. Character-by-Character Breakdown:");
    println!("   Pos | Hex  | ASCII | Meaning");
    println!("   ----|------|-------|--------");

    for (i, &byte) in torrent_bytes.iter().take(36).enumerate() {
        let ascii_char = if byte.is_ascii_graphic() || byte == b' ' {
            format!("'{}'", byte as char)
        } else {
            "?".to_string()
        };

        let meaning = match i {
            0 => "Dictionary start",
            1 => "Key length digit '8'",
            2 => "Length separator ':'",
            3..=10 => "Key 'announce'",
            11 => "Value length '2' (first digit)",
            12 => "Value length '2' (second digit)",
            13 => "Value separator ':'",
            14..=35 => "Announce URL value",
            _ => "Next field...",
        };

        println!(
            "   {:3} | {:02x}   | {:5} | {}",
            i, byte, ascii_char, meaning
        );
    }

    println!("\n7. Complete Structure Visualization:");
    println!("   d8:announce22:udp://opentor.net:696913:announce-list...");
    println!("   ^          ^                      ^");
    println!("   |          |                      |");
    println!("   |          |                      +-- Next key starts");
    println!("   |          +-- Announce URL value");
    println!("   +-- Dictionary containing the torrent metadata");

    println!("\n8. More Bencode Examples:");
    println!("   Simple string:  '4:spam' → 'spam'");
    println!("   Number:         'i42e' → 42");
    println!("   Empty list:     'le' → []");
    println!("   List:           'l4:spam4:eggse' → ['spam', 'eggs']");
    println!("   Dictionary:     'd3:cow3:moo4:spam4:eggse' → {{'cow': 'moo', 'spam': 'eggs'}}");

    println!("\n9. Why This Format?");
    println!("   ✓ Simple to parse (no escaping needed)");
    println!("   ✓ Compact representation");
    println!("   ✓ Language agnostic");
    println!("   ✓ Self-describing (lengths are specified)");
    println!("   ✓ Used throughout BitTorrent protocol");

    println!("\n10. In a Real Torrent File:");
    println!("    The 'announce' field tells BitTorrent clients where to find");
    println!("    the tracker server that coordinates peers for this torrent.");

    // Try to extract just the announce URL manually
    if let Some(announce_url) = extract_announce_manually(&torrent_bytes) {
        println!("    ");
        println!("    ✓ Extracted announce URL: {}", announce_url);
        println!(
            "    ✓ This is encoded as: {}:{}",
            announce_url.len(),
            announce_url
        );
        println!(
            "    ✓ Total bytes used: {} (key) + {} (value) = {} bytes",
            "8:announce".len(),
            format!("{}:{}", announce_url.len(), announce_url).len(),
            "8:announce".len() + format!("{}:{}", announce_url.len(), announce_url).len()
        );
    }

    println!("\n11. Key Insight:");
    println!("    Every string in bencode is prefixed with its length, making");
    println!("    parsing unambiguous and efficient - no need to scan for");
    println!("    terminators or handle escape sequences!");

    Ok(())
}

fn extract_announce_manually(bytes: &[u8]) -> Option<String> {
    // Look for the pattern: d8:announce<length>:
    let mut pos = 0;

    // Skip 'd8:announce'
    if bytes.len() > 11 && &bytes[0..11] == b"d8:announce" {
        pos = 11;

        // Read the length
        let mut length_str = String::new();
        while pos < bytes.len() && bytes[pos].is_ascii_digit() {
            length_str.push(bytes[pos] as char);
            pos += 1;
        }

        if let Ok(length) = length_str.parse::<usize>() {
            // Skip the colon
            if pos < bytes.len() && bytes[pos] == b':' {
                pos += 1;

                // Extract the announce URL
                if pos + length <= bytes.len() {
                    if let Ok(url) = String::from_utf8(bytes[pos..pos + length].to_vec()) {
                        return Some(url);
                    }
                }
            }
        }
    }

    None
}
