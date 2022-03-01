use std::io;
use crate::ReadError;
use crate::Section;
use crate::linereader::LineReader;

/// A parser for the text of an EditorConfig file.
///
/// This struct wraps any [std::io::BufRead]
/// and parses the prelude and zero or more sections from it.
pub struct EcReader<R: io::BufRead> {
	/// Incidates if a `root = true` line was found in the prelude.
	pub is_root: bool,
	eof: bool,
	reader: LineReader<R>
}

impl<R: io::Read> EcReader<io::BufReader<R>> {
	/// See [EcReader::new].
	pub fn new_buffered(source: R) -> Result<EcReader<io::BufReader<R>>, ReadError> {
		Self::new(io::BufReader::new(source))
	}
}

impl<R: io::BufRead> EcReader<R> {
	/// Constructs a new [EcReader] and reads the prelude from the provided source.
	///
	/// Returns `Ok` if the prelude was read successfully,
	/// otherwise returns `Err` with the error that occurred during reading.
	pub fn new(buf_source: R) -> Result<EcReader<R>, ReadError> {
		let mut reader = LineReader::new(buf_source);
		let (is_root, eof) = reader.read_prelude()?;
		Ok(EcReader {is_root, reader, eof})
	}

	/// Returns `true` if there may be another section to read.
	pub fn has_more(&self) -> bool {
		self.eof
	}

	/// Reads a [Section] from the internal source.
	pub fn read_section(&mut self) -> Result<Section, ReadError> {
		if !self.eof {
			match self.reader.read_section() {
				Ok((section, eof)) => {
					self.eof = eof;
					Ok(section)
				}
				Err(e) => {
					self.eof = true;
					Err(e)
				}
			}
		} else {
			Err(ReadError::Eof)
		}
	}
}

impl<R: io::BufRead> Iterator for EcReader<R> {
	type Item = Result<Section, ReadError>;
	fn next(&mut self) -> Option<Self::Item> {
		match self.read_section() {
			Ok(r)               => Some(Ok(r)),
			Err(ReadError::Eof) => None,
			Err(e)              => Some(Err(e))
		}
	}
}

impl<R: io::BufRead> std::iter::FusedIterator for EcReader<R> {}


impl<R: io::BufRead> crate::PropertiesSource for &mut EcReader<R> {
	fn apply_to(self, props: &mut crate::Properties, path: impl AsRef<std::path::Path>) {
		let path = path.as_ref();
		for section in self.flatten() {
				section.apply_to(props, path)
		}
	}
}
