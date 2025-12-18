// SPDX-License-Identifier: MPL-2.0
//! XMP metadata extraction and writing for Dublin Core fields.
//!
//! This module handles reading and writing XMP (Extensible Metadata Platform) data
//! embedded in image files. It focuses on Dublin Core elements commonly used for
//! describing creative works.
//!
//! Supported Dublin Core elements:
//! - dc:title - Title of the work
//! - dc:creator - Creator/author
//! - dc:description - Description
//! - dc:subject - Keywords/tags
//! - dc:rights - Copyright/license

use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Dublin Core metadata extracted from XMP.
#[derive(Debug, Clone, Default)]
pub struct DublinCoreMetadata {
    pub title: Option<String>,
    pub creator: Option<String>,
    pub description: Option<String>,
    pub subject: Option<Vec<String>>,
    pub rights: Option<String>,
}

/// XMP namespace prefixes used in parsing.
const XMP_MARKER: &[u8] = b"http://ns.adobe.com/xap/1.0/";
const DC_NS: &str = "http://purl.org/dc/elements/1.1/";

/// Extract XMP data from a JPEG file.
///
/// XMP in JPEG is stored in APP1 segments with the marker "http://ns.adobe.com/xap/1.0/".
pub fn extract_xmp_from_jpeg<P: AsRef<Path>>(path: P) -> Option<DublinCoreMetadata> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);

    // Find and extract XMP segment
    let xmp_data = find_jpeg_xmp_segment(&mut reader)?;

    // Parse XMP XML
    parse_xmp_xml(&xmp_data)
}

/// Find XMP APP1 segment in JPEG file.
fn find_jpeg_xmp_segment<R: Read + Seek>(reader: &mut R) -> Option<Vec<u8>> {
    let mut marker = [0u8; 2];

    // Check JPEG magic number
    reader.read_exact(&mut marker).ok()?;
    if marker != [0xFF, 0xD8] {
        return None; // Not a JPEG
    }

    // Scan through segments
    loop {
        // Read segment marker
        reader.read_exact(&mut marker).ok()?;

        if marker[0] != 0xFF {
            return None; // Invalid JPEG structure
        }

        match marker[1] {
            0xD9 => return None, // End of image, no XMP found
            0xD8 => continue,    // Start of image (shouldn't happen here)
            0x00 => continue,    // Stuffed byte
            0xE1 => {
                // APP1 segment - could be EXIF or XMP
                let mut len_bytes = [0u8; 2];
                reader.read_exact(&mut len_bytes).ok()?;
                let segment_len = u16::from_be_bytes(len_bytes) as usize;

                if segment_len < 2 {
                    return None;
                }

                let data_len = segment_len - 2;
                let mut segment_data = vec![0u8; data_len];
                reader.read_exact(&mut segment_data).ok()?;

                // Check if this is XMP (starts with XMP marker + null byte)
                if segment_data.len() > XMP_MARKER.len() + 1
                    && segment_data.starts_with(XMP_MARKER)
                    && segment_data[XMP_MARKER.len()] == 0
                {
                    // Extract XMP XML (after marker + null byte)
                    let xmp_start = XMP_MARKER.len() + 1;
                    return Some(segment_data[xmp_start..].to_vec());
                }
            }
            marker_type => {
                // Skip other segments
                if (0xD0..=0xD7).contains(&marker_type) {
                    // RST markers have no length
                    continue;
                }

                let mut len_bytes = [0u8; 2];
                reader.read_exact(&mut len_bytes).ok()?;
                let segment_len = u16::from_be_bytes(len_bytes) as usize;

                if segment_len < 2 {
                    return None;
                }

                // Skip segment content
                reader
                    .seek(SeekFrom::Current((segment_len - 2) as i64))
                    .ok()?;
            }
        }
    }
}

/// Parse XMP XML and extract Dublin Core metadata.
fn parse_xmp_xml(xmp_data: &[u8]) -> Option<DublinCoreMetadata> {
    let mut metadata = DublinCoreMetadata::default();
    let mut reader = Reader::from_reader(xmp_data);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut current_element: Option<String> = None;
    let mut in_rdf_seq = false;
    let mut current_subjects: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let local_name = e.local_name();
                let name = String::from_utf8_lossy(local_name.as_ref()).to_string();

                // Check if we're in Dublin Core namespace
                let is_dc = e.attributes().any(|attr| {
                    if let Ok(a) = attr {
                        a.key.as_ref() == b"xmlns:dc"
                            && a.unescape_value().map(|v| v == DC_NS).unwrap_or(false)
                    } else {
                        false
                    }
                });

                // Track DC elements or elements with dc: prefix
                let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if element_name.starts_with("dc:") || is_dc {
                    current_element = Some(name.clone());
                }

                // Track rdf:Seq for subject arrays
                if name == "Seq" || element_name.ends_with(":Seq") {
                    in_rdf_seq = true;
                }

                // Handle rdf:li items in sequences
                if (name == "li" || element_name.ends_with(":li")) && in_rdf_seq {
                    // Will capture text in Text event
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref element) = current_element {
                    let text = e.decode().ok()?.trim().to_string();
                    if !text.is_empty() {
                        match element.as_str() {
                            "title" => metadata.title = Some(text),
                            "creator" => metadata.creator = Some(text),
                            "description" => metadata.description = Some(text),
                            "rights" => metadata.rights = Some(text),
                            "li" if in_rdf_seq => {
                                current_subjects.push(text);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if name == "Seq" || element_name.ends_with(":Seq") {
                    in_rdf_seq = false;
                }

                // When closing subject element, save collected subjects
                if (name == "subject" || element_name == "dc:subject")
                    && !current_subjects.is_empty()
                {
                    metadata.subject = Some(current_subjects.clone());
                    current_subjects.clear();
                }

                if element_name.starts_with("dc:") {
                    current_element = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    // Return None if no DC metadata found
    if metadata.title.is_none()
        && metadata.creator.is_none()
        && metadata.description.is_none()
        && metadata.subject.is_none()
        && metadata.rights.is_none()
    {
        return None;
    }

    Some(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_xmp_xml_extracts_dublin_core() {
        let xmp = r#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
    <rdf:Description rdf:about=""
        xmlns:dc="http://purl.org/dc/elements/1.1/">
      <dc:title>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">My Photo Title</rdf:li>
        </rdf:Alt>
      </dc:title>
      <dc:creator>
        <rdf:Seq>
          <rdf:li>John Doe</rdf:li>
        </rdf:Seq>
      </dc:creator>
      <dc:description>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">A beautiful sunset</rdf:li>
        </rdf:Alt>
      </dc:description>
      <dc:subject>
        <rdf:Bag>
          <rdf:li>sunset</rdf:li>
          <rdf:li>nature</rdf:li>
          <rdf:li>landscape</rdf:li>
        </rdf:Bag>
      </dc:subject>
      <dc:rights>
        <rdf:Alt>
          <rdf:li xml:lang="x-default">© 2024 John Doe</rdf:li>
        </rdf:Alt>
      </dc:rights>
    </rdf:Description>
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let metadata = parse_xmp_xml(xmp.as_bytes()).expect("Should parse XMP");

        assert_eq!(metadata.title, Some("My Photo Title".to_string()));
        assert_eq!(metadata.creator, Some("John Doe".to_string()));
        assert_eq!(metadata.description, Some("A beautiful sunset".to_string()));
        assert_eq!(metadata.rights, Some("© 2024 John Doe".to_string()));
    }

    #[test]
    fn parse_xmp_xml_returns_none_for_empty() {
        let xmp = br#"<?xpacket begin="" id="W5M0MpCehiHzreSzNTczkc9d"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
  <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  </rdf:RDF>
</x:xmpmeta>
<?xpacket end="w"?>"#;

        let result = parse_xmp_xml(xmp);
        assert!(result.is_none());
    }

    #[test]
    fn dublin_core_metadata_default() {
        let metadata = DublinCoreMetadata::default();
        assert!(metadata.title.is_none());
        assert!(metadata.creator.is_none());
        assert!(metadata.description.is_none());
        assert!(metadata.subject.is_none());
        assert!(metadata.rights.is_none());
    }
}
