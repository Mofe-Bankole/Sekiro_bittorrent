use anyhow::{Error, anyhow};
use bytes::{Buf, Bytes};

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
// Bencode struct
pub enum BencodeValue {
    Bytes(Bytes),
    Integer(i64),
    List(Vec<BencodeValue>),
    Dictionary(Vec<BencodeValue>)
}

impl BencodeValue {
    pub fn decode_from_reader(bytes: &[u8]) -> Result<BencodeValue, Error> {
        let mut reader = Bytes::from(bytes.to_vec());

        if reader.has_remaining(){
            anyhow!("PANICED CAUSE THERES SOMETHING REMAINING");
        }

        match reader.chunk()[0] {
            b'l' => Self::decode_list(&mut reader),
            b'i' => Self::decode_integer(&mut reader),
            b'd' => Self::decode_dictionary(&mut reader),
            b'0'..=b'9' => Self::decode_bytes(&mut reader),
            other => Err(anyhow!("Unexpected byte: {}", other)),
        };


        Ok(())
    }

    pub fn decode(data: &[u8]) -> Result<BencodeValue, Error> {
        let reader = Bytes::from(data.to_vec());
        let value = Self::decode_from_reader(&reader)?;

        if reader.has_remaining() {
            return Err(anyhow::anyhow!(
                "Leftover data after decoding Bencode value. Remaining: {} bytes",
                reader.remaining()
            ));
        }
        Ok(value);
    }

    // Decoding Functionality for Lists
    fn decode_list(reader: &mut Bytes) -> Result<BencodeValue, Error> {
        reader.advance(1);
        let mut list = Vec::new();

        while reader.has_remaining() && reader.chunk()[0] != b'e' {
            list.push(Self::decode_from_reader(reader)?);
        }

        if !reader.has_remaining() || reader.chunk()[0] != b'e' {
            return Err(anyhow!("List not terminated by 'e'"));
        }
        reader.advance(1); // Consume 'e'
        Ok(BencodeValue::List(list))
    }

    // Decoding Functionality for Integers
    fn decode_integer(reader: &mut Bytes) -> Result<BencodeValue, Error> {
        reader.advance(1);
        let mut number_bytes = Vec::new();

        while reader.has_remaining() && !reader.chunk().is_empty() {
            let byte = reader.chunk()[0];

            if byte == b'e' {
                break;
            }

            number_bytes.push(byte);
            reader.advance(1);
        }

        if !reader.has_remaining() || reader.chunk()[0] != b'e' {
            return Err(anyhow!("Integer not terminated by 'e'".to_string()));
        }
        reader.advance(1);

        // Convert bytes to string
        let number_str = String::from_utf8(number_bytes).map_err(|_| anyhow!("Invalid UTF-8 in integer"))?;

        // Parse to i64
        let number = number_str
            .parse::<i64>()
            .map_err(|_| anyhow!("Invalid integer format: {}", number_str))?;

        Ok(BencodeValue::Integer(number))
    }

    // Decoding Functionality for Dictionaries
    fn decode_dictionary(reader: &mut Bytes) -> Result<BencodeValue, Error> {
        reader.advance(1);
        let mut dictionary = Vec::new();

        while reader.has_remaining() {
            let key = Self::decode_from_reader(reader)?;
            let value = Self::decode_from_reader(reader)?;

            dictionary.push(key);
            dictionary.push(value);
        }

        if !reader.has_remaining() || reader.chunk()[0] != b'e' {
            return Err(anyhow!("Dictionary not terminated by 'e'".to_string()));
        }
        reader.advance(1);

        Ok(BencodeValue::Dictionary(dictionary))
    }

    fn decode_bytes(reader :&mut Bytes) -> Result<BencodeValue , Error>{
        reader.advance(1);
        let length_bytes = Vec::new();

        while reader.has_remaining() && !reader.chunk().is_empty(){
            let byte = reader.chunk()[0];

            if byte == b':'{
                break;
            }

            if !byte.is_ascii_digit(){
                return Err(anyhow!("Invalid Bytes Length Prefix"));
            }
            length_bytes.push(byte);
            reader.advance(1);
        }

        if !reader.has_remaining() || reader.chunk()[0] != b':' {
            return Err(anyhow!("Expected Values not Found"))
        }
        reader.advance(1);

        let length = String::from_utf8(length_bytes)?;
        let length: usize = length_str.parse()?;

          if reader.remaining() < length {
              return Err(anyhow!("Not enough bytes to read string"));
          }

          let value = reader.copy_to_bytes(length);
          Ok(BencodeValue::Bytes(value))
    }
}
