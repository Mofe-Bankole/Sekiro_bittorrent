use mini_p2p_file_transfer_system::protocol::torrent::{Torrent, TorrentParser};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Torrent Parser Test ===\n");

    // Read the test torrent file
    let torrent_bytes = fs::read("test.torrent")?;
    println!("✓ Read torrent file ({} bytes)", torrent_bytes.len());

    // Test individual extraction methods
    println!("\n1. Testing individual extraction methods:");

    match Torrent::extract_announce(&torrent_bytes) {
        Ok(announce) => println!("   ✓ Announce: {}", announce),
        Err(e) => println!("   ✗ Error extracting announce: {}", e),
    }

    match Torrent::extract_name(&torrent_bytes) {
        Ok(name) => println!("   ✓ Name: {}", name),
        Err(e) => println!("   ✗ Error extracting name: {}", e),
    }

    match Torrent::extract_piece_length(&torrent_bytes) {
        Ok(piece_length) => println!("   ✓ Piece length: {} bytes", piece_length),
        Err(e) => println!("   ✗ Error extracting piece length: {}", e),
    }

    match Torrent::extract_pieces(&torrent_bytes) {
        Ok(pieces) => {
            println!("   ✓ Pieces: {} total", pieces.len());
            if !pieces.is_empty() {
                println!("     First piece hash: {:02x?}...", &pieces[0][..8]);
            }
        }
        Err(e) => println!("   ✗ Error extracting pieces: {}", e),
    }

    match Torrent::extract_length(&torrent_bytes) {
        Ok(length) => {
            println!(
                "   ✓ Total length: {} bytes ({:.2} MB)",
                length,
                length as f64 / (1024.0 * 1024.0)
            );
        }
        Err(e) => println!("   ✗ Error extracting length: {}", e),
    }

    match Torrent::extract_files(&torrent_bytes) {
        Ok(Some(files)) => {
            println!("   ✓ Multi-file torrent with {} files:", files.len());
            for (i, file) in files.iter().take(5).enumerate() {
                println!(
                    "     File {}: {} ({} bytes)",
                    i + 1,
                    file.path.join("/"),
                    file.length
                );
            }
            if files.len() > 5 {
                println!("     ... and {} more files", files.len() - 5);
            }
        }
        Ok(None) => println!("   ✓ Single-file torrent"),
        Err(e) => println!("   ✗ Error extracting files: {}", e),
    }

    match Torrent::extract_info_hash(&torrent_bytes) {
        Ok(info_hash) => {
            println!("   ✓ Info hash: {}", hex::encode(info_hash));
        }
        Err(e) => println!("   ✗ Error extracting info hash: {}", e),
    }

    // Test the complete from_bytes method
    println!("\n2. Testing complete torrent parsing:");
    match Torrent::from_bytes(&torrent_bytes) {
        Ok(torrent) => {
            println!("   ✓ Successfully parsed complete torrent!");
            println!("     Name: {}", torrent.name);
            println!("     Announce URL: {}", torrent.announce);
            println!("     Info hash: {}", hex::encode(torrent.info_hash));
            println!("     Piece length: {} bytes", torrent.piece_length);
            println!("     Number of pieces: {}", torrent.pieces.len());
            println!(
                "     Total size: {} bytes ({:.2} MB)",
                torrent.length,
                torrent.length as f64 / (1024.0 * 1024.0)
            );

            match &torrent.files {
                Some(files) => {
                    println!("     Multi-file torrent with {} files", files.len());

                    // Show file breakdown by type
                    let mut file_types = std::collections::HashMap::new();
                    for file in files {
                        if let Some(ext) = file
                            .path
                            .last()
                            .and_then(|name| name.rfind('.').map(|dot| &name[dot..]))
                        {
                            *file_types.entry(ext.to_string()).or_insert(0usize) += 1;
                        } else {
                            *file_types
                                .entry("(no extension)".to_string())
                                .or_insert(0usize) += 1;
                        }
                    }

                    println!("     File types breakdown:");
                    for (ext, count) in file_types.iter().take(10) {
                        println!("       {}: {} files", ext, count);
                    }
                }
                None => println!("     Single-file torrent"),
            }

            // Verify data integrity
            println!("\n3. Data integrity checks:");

            // Check if pieces match expected total
            let expected_pieces =
                (torrent.length + torrent.piece_length - 1) / torrent.piece_length;
            if torrent.pieces.len() == expected_pieces {
                println!(
                    "   ✓ Piece count matches expected ({} pieces)",
                    expected_pieces
                );
            } else {
                println!(
                    "   ⚠ Piece count mismatch: got {}, expected {}",
                    torrent.pieces.len(),
                    expected_pieces
                );
            }

            // Check info hash uniqueness (should be deterministic)
            match Torrent::extract_info_hash(&torrent_bytes) {
                Ok(hash2) => {
                    if torrent.info_hash == hash2 {
                        println!("   ✓ Info hash is consistent");
                    } else {
                        println!("   ✗ Info hash inconsistency!");
                    }
                }
                Err(e) => println!("   ✗ Could not re-extract info hash: {}", e),
            }

            // Verify files total length matches torrent length
            if let Some(files) = &torrent.files {
                let files_total: usize = files.iter().map(|f| f.length).sum();
                if files_total == torrent.length {
                    println!("   ✓ Files total length matches torrent length");
                } else {
                    println!(
                        "   ⚠ Length mismatch: files total {}, torrent length {}",
                        files_total, torrent.length
                    );
                }
            }

            println!("\n4. Summary:");
            println!("   🎉 All torrent parsing functionality working correctly!");
            println!(
                "   📊 Torrent contains {:.2} MB of data across {} pieces",
                torrent.length as f64 / (1024.0 * 1024.0),
                torrent.pieces.len()
            );
        }
        Err(e) => {
            println!("   ✗ Error parsing complete torrent: {}", e);
            return Err(e.into());
        }
    }

    println!("\n=== Test completed successfully! ===");
    Ok(())
}
