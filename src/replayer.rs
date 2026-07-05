use {
  crate::{
    SEGMENT_NAME_PREFIX, SegmentID, checkpoint_file::CheckpointFile, decode_entry, error::Result,
    proto::WalEntry, read_entry_encoding_size
  },
  std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    os::unix::fs::MetadataExt,
    path::PathBuf
  }
};

pub struct Replayer {
  dir: PathBuf,

  current_segment_id:     SegmentID,
  current_segment_reader: Option<File>
}

impl Replayer {
  /// Constructs an instance of the WAL replayer.
  /// We assume that a latest checkpoint already exists.
  pub fn new(dir: PathBuf) -> Result<Self> {
    // Read information about the latest checkpoint from the .checkpoint file.
    let (current_segment_id, checkpoint_entry_lsn) = CheckpointFile::new(&dir).read()?;

    // Create the segment reader.

    let current_segment_path = dir.join(format!("{SEGMENT_NAME_PREFIX}{current_segment_id}"));

    let mut current_segment_reader = File::options().read(true).open(current_segment_path)?;

    current_segment_reader.seek(SeekFrom::Start(0))?;

    // And seek to the end of the entry which represents the latest checkpoint.
    loop {
      let current_entry = read_entry(&mut current_segment_reader)?;
      if current_entry.lsn == checkpoint_entry_lsn {
        break;
      }
    }

    Ok(Self { dir,

              current_segment_id,
              current_segment_reader: Some(current_segment_reader) })
  }

  /// Get the next entry, if exists.
  pub fn try_next(&mut self) -> Result<Option<WalEntry>> {
    loop {
      // Try getting the current segment reader.
      let current_segment_reader = match self.current_segment_reader {
        // There are no more segments to read.
        // So, we don't need to do anything further.
        | None => return Ok(None),

        | Some(ref mut current_segment_reader) => current_segment_reader
      };

      // Get the seek position of the current segment reader, and, check whether it's at the EOF :
      // which means there are no more entries to be read from the current segment, and we need to
      // look if there is a next segment to read.

      let seek_position = current_segment_reader.stream_position()?;

      let current_segment_size = current_segment_reader.metadata()?.size();

      if seek_position == current_segment_size {
        let next_segment_id = self.current_segment_id + 1;
        let next_segment_path = self.dir.join(format!("{SEGMENT_NAME_PREFIX}{next_segment_id}"));

        let mut next_segment_reader = match File::options().read(true).open(next_segment_path) {
          // The next segment doesn't exist.
          // So, we don't need to do anything further.
          | Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),

          | result => result
        }?;

        // There is a next segment to read.

        // Seek to it's beginning.
        next_segment_reader.seek(SeekFrom::Start(0))?;

        // And make it the current segment. We're going to process it in the next iteration.
        self.current_segment_id = next_segment_id;
        self.current_segment_reader = Some(next_segment_reader);

        continue;
      }

      // At this point, we're sure that there's an entry to be read in the current segment.
      let current_entry = read_entry(current_segment_reader)?;
      return Ok(Some(current_entry));
    }
  }
}

/// Expects that the segment reader is currently seeking at the beginning of a length prefixed
/// entry, i.e., entry encoding size followed by the actual entry encoding.
/// The entry is read and returned.
/// The segment reader is seeked to the end of the entry.
fn read_entry(segment_reader: &mut File) -> Result<WalEntry> {
  // Read the entry encoding size.
  let entry_encoding_size = read_entry_encoding_size(segment_reader)?;

  // Read and decode the entry encoding.

  let mut entry_encoding = vec![0u8; entry_encoding_size as usize];
  segment_reader.read_exact(&mut entry_encoding)?;

  let entry = decode_entry(&entry_encoding)?;

  Ok(entry)
}
