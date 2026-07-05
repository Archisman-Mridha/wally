use {
  crate::{
    CHECKPOINT_FILE_NAME, LSN, SegmentID,
    error::{Error, Result}
  },
  std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf}
  }
};

type LatestCheckpointDetails = (SegmentID, LSN);

/**
  Represents the .checkpoint file , which holds information about the latest checkpoint entry, in
  this format :

                                    {segment ID}:{entry LSN}
*/
pub struct CheckpointFile {
  path: PathBuf
}

impl CheckpointFile {
  pub fn new(wal_dir: &Path) -> Self { Self { path: wal_dir.join(CHECKPOINT_FILE_NAME) } }

  /// Update the checkpoint file with the provided latest checkpoint details.
  pub fn update(&self, latest_checkpoint_details: LatestCheckpointDetails) -> Result<()> {
    let (segment_id, entry_lsn) = latest_checkpoint_details;

    let latest_checkpoint_information = format!("{}:{}", segment_id, entry_lsn);

    let mut checkpoint_file_updater =
      File::options().create(true).write(true).truncate(true).open(&self.path)?;

    checkpoint_file_updater.write_all(latest_checkpoint_information.as_bytes())?;

    checkpoint_file_updater.sync_data()?;

    Ok(())
  }

  /// Read the latest checkpoint details.
  pub fn read(&self) -> Result<LatestCheckpointDetails> {
    let latest_checkpoint_information = fs::read_to_string(&self.path)?;

    let (segment_id, entry_lsn) =
      latest_checkpoint_information.trim().split_once(":").ok_or(Error::InvalidCheckpointFile)?;

    let segment_id = segment_id.parse::<SegmentID>().map_err(|_| Error::InvalidCheckpointFile)?;

    let entry_lsn = entry_lsn.parse::<u32>().map_err(|_| Error::InvalidCheckpointFile)?;

    Ok((segment_id, entry_lsn))
  }
}
