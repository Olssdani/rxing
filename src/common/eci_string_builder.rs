/*
 * Copyright 2022 ZXing authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// package com.google.zxing.common;

// import com.google.zxing.FormatException;

// import java.nio.charset.Charset;
// import java.nio.charset.StandardCharsets;

use std::fmt;

use super::{CharacterSet, Eci};

/**
 * Class that converts a sequence of ECIs and bytes into a string
 *
 * @author Alex Geller
 */
#[derive(Default)]
pub struct ECIStringBuilder {
    is_eci: bool,
    eci_result: Option<String>,
    bytes: Vec<u8>,
    eci_positions: Vec<(Eci, usize, usize)>, // (Eci, start, end)
}

impl ECIStringBuilder {
    pub fn with_capacity(initial_capacity: usize) -> Self {
        Self {
            eci_result: None,
            bytes: Vec::with_capacity(initial_capacity),
            eci_positions: Vec::default(),
            is_eci: false,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /**
     * Appends {@code value} as a byte value
     *
     * @param value character whose lowest byte is to be appended
     */
    pub fn append_char(&mut self, value: char) {
        self.eci_result = None;
        self.bytes.push(value as u8);
    }

    /**
     * Appends {@code value} as a byte value
     *
     * @param value byte to append
     */
    pub fn append_byte(&mut self, value: u8) {
        self.eci_result = None;
        self.bytes.push(value)
    }

    pub fn append_bytes(&mut self, value: &[u8]) {
        self.eci_result = None;
        self.bytes.extend_from_slice(value)
    }

    /**
     * Appends the characters in {@code value} as bytes values
     *
     * @param value string to append
     */
    pub fn append_string(&mut self, value: &str) {
        if !value.is_ascii() {
            self.append_eci(Eci::UTF8);
        }
        self.eci_result = None;
        self.bytes.extend_from_slice(value.as_bytes());
    }

    /**
     * Append the string repesentation of {@code value} (short for {@code append(String.valueOf(value))})
     *
     * @param value int to append as a string
     */
    pub fn append(&mut self, value: i32) {
        self.append_string(&format!("{value}"));
    }

    /**
     * Appends ECI value to output.
     *
     * @param value ECI value to append, as an int
     * @throws FormatException on invalid ECI value
     */
    pub fn append_eci(&mut self, eci: Eci) {
        self.eci_result = None;
        if !self.is_eci && eci != Eci::ISO8859_1 {
            self.is_eci = true;
        }

        if self.is_eci {
            if let Some(last) = self.eci_positions.last_mut() {
                last.2 = self.bytes.len()
            }

            self.eci_positions.push((eci, self.bytes.len(), 0));
        }
    }

    /// Finishes encoding anything in the buffer using the current ECI and resets.
    ///
    /// This function can panic
    pub fn encodeCurrentBytesIfAny(&self) -> String {
        let mut encoded_string = String::with_capacity(self.bytes.len());
        // First encode the first set
        let (_, end, _) =
            *self
                .eci_positions
                .first()
                .unwrap_or(&(Eci::ISO8859_1, self.bytes.len(), 0));

        encoded_string.push_str(
            &Self::encode_segment(&self.bytes[0..end], Eci::ISO8859_1).unwrap_or_default(),
        );

        // If there are more sets, encode each of them in turn
        for (eci, eci_start, eci_end) in &self.eci_positions {
            // let (_,end) = *self.eci_positions.first().unwrap_or(&(*eci, self.bytes.len()));
            let end = if *eci_end == 0 {
                self.bytes.len()
            } else {
                *eci_end
            };
            encoded_string.push_str(
                &Self::encode_segment(&self.bytes[*eci_start..end], *eci).unwrap_or_default(),
            );
        }

        // Return the result
        encoded_string
    }

    fn encode_segment(bytes: &[u8], eci: Eci) -> Option<String> {
        let mut encoded_string = String::with_capacity(bytes.len());
        if ![Eci::Binary, Eci::Unknown].contains(&eci) {
            if eci == Eci::UTF8 {
                if !bytes.is_empty() {
                    encoded_string.push_str(&CharacterSet::UTF8.decode(bytes).ok()?);
                } else {
                    return None;
                }
            } else if !bytes.is_empty() {
                encoded_string.push_str(&CharacterSet::from(eci).decode(bytes).ok()?);
            } else {
                return None;
            }
        } else {
            for byte in bytes {
                encoded_string.push(char::from(*byte))
            }
        }

        if encoded_string.is_empty() {
            None
        } else {
            Some(encoded_string)
        }
    }

    /**
     * Appends the characters from {@code value} (unlike all other append methods of this class who append bytes)
     *
     * @param value characters to append
     */
    pub fn appendCharacters(&mut self, value: &str) {
        self.append_string(value);
    }

    /**
     * Short for {@code toString().length()} (if possible, use {@link #isEmpty()} instead)
     *
     * @return length of string representation in characters
     */
    pub fn len(&mut self) -> usize {
        self.bytes.len()
    }

    /**
     * @return true iff nothing has been appended
     */
    pub fn is_empty(&mut self) -> bool {
        self.bytes.is_empty()
    }

    pub fn build_result(mut self) -> Self {
        self.eci_result = Some(self.encodeCurrentBytesIfAny());

        self
    }
}

impl fmt::Display for ECIStringBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(res) = &self.eci_result {
            write!(f, "{res}")
        } else {
            write!(f, "{}", self.encodeCurrentBytesIfAny())
        }
    }
}
