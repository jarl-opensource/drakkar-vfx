use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use gix::Repository;
use gix::bstr::BStr;
use gix::diff::index::Change;
use gix::status::{Item, index_worktree};
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScmStatus
{
    Unknown,
    DoesNotExist,
    New,
    Modified,
    Unmodified,
    Loading,
}

/// S
pub struct Scm
{
    inner:          Arc<Mutex<ScmInner>>,
    polling_task:   Option<std::thread::JoinHandle<()>>,
    command_sender: Option<Sender<PollingCommand>>,
}

impl Scm
{
    /// Creates a new SCM instance with repository discovery and initial file list
    pub fn new(path: &Path, file_paths: Vec<PathBuf>) -> Self
    {
        let repo_path = path.to_path_buf();
        let inner = Arc::new(Mutex::new(ScmInner::new(repo_path)));

        let mut scm = Self {
            inner:          inner.clone(),
            polling_task:   None,
            command_sender: None,
        };

        scm.update_tracked_files(file_paths.clone());
        scm.start_polling();
        scm
    }

    /// Check if SCM repository is detected
    pub fn is_detected(&self) -> bool
    {
        if let Ok(inner) = self.inner.lock() {
            inner.repository.is_some()
        } else {
            false
        }
    }

    /// Get the path where SCM was searched
    pub fn path(&self) -> String
    {
        if let Ok(inner) = self.inner.lock() {
            inner.repo_path.to_string_lossy().to_string()
        } else {
            String::new()
        }
    }

    pub fn get_scm_status(&self, file_path: &Path) -> ScmStatus
    {
        let path_key = file_path.to_string_lossy().to_string();
        if let Ok(inner) = self.inner.lock() {
            if let Some(cached) = inner.statuses.get(&path_key) {
                return cached.scm_status;
            }
        }
        ScmStatus::Unknown
    }

    /// Check if a file has been modified compared to the committed version
    pub fn is_file_modified(&self, file_path: &Path) -> bool
    {
        self.get_scm_status(file_path) == ScmStatus::Modified
    }

    /// Update the file list for polling
    pub fn update_tracked_files(&mut self, file_paths: Vec<PathBuf>)
    {
        if let Ok(mut inner) = self.inner.lock() {
            inner.refresh_paths(file_paths);
        }
    }

    /// Refresh the SCM state by re-discovering the repository
    pub fn update_repository(&mut self)
    {
        if let Ok(mut inner) = self.inner.lock() {
            inner.refresh_repository();
        }
    }

    /// Stop background polling task
    pub fn stop_polling(&mut self)
    {
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(PollingCommand::Stop);
        }
        if let Some(handle) = self.polling_task.take() {
            let _ = handle.join();
        }
        self.command_sender = None;
    }

    // ====================
    // Private methods
    // ====================

    /// Start background polling for SCM status updates
    fn start_polling(&mut self)
    {
        self.stop_polling();
        let inner = self.inner.clone();
        let (command_sender, command_receiver) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            Self::polling_worker(inner, command_receiver);
        });
        self.polling_task = Some(handle);
        self.command_sender = Some(command_sender);
    }

    /// Main polling worker function
    fn polling_worker(inner: Arc<Mutex<ScmInner>>, command_receiver: Receiver<PollingCommand>)
    {
        loop {
            // Check for commands
            if let Ok(command) = command_receiver.try_recv() {
                match command {
                    PollingCommand::Stop => {
                        break;
                    }
                }
            }

            if let Ok(inner_guard) = inner.lock() {
                if let Some(repository) = &inner_guard.repository {
                    let paths = inner_guard.tracked_paths.clone();
                    let repo_path = repository.path().to_path_buf();
                    drop(inner_guard); // Release the lock
                    if let Ok(repository) = gix::open(&repo_path) {
                        let new_statuses =
                            ScmInner::get_scm_statuses_for_files(&repository, &paths);
                        if let Ok(mut inner_guard) = inner.lock() {
                            inner_guard.statuses = new_statuses;
                        }
                    }
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}

impl Drop for Scm
{
    fn drop(&mut self)
    {
        self.stop_polling();
    }
}

#[derive(Clone, Debug)]
pub struct CachedStatus
{
    pub scm_status:  ScmStatus,
    pub last_update: Instant,
}

impl CachedStatus
{
    pub fn new(scm_status: ScmStatus) -> Self
    {
        Self {
            scm_status,
            last_update: Instant::now(),
        }
    }
}

/// Commands that can be sent to the polling thread
enum PollingCommand
{
    Stop,
}

/// Internal shared state containing repository, tracked paths, and statuses
struct ScmInner
{
    /// Git repository instance
    repository:    Option<Repository>,
    /// Currently tracked file paths
    tracked_paths: Vec<PathBuf>,
    /// Cached statuses for all tracked files
    statuses:      HashMap<String, CachedStatus>,
    /// Repository path for re-discovery
    repo_path:     PathBuf,
}

impl ScmInner
{
    fn new(repo_path: PathBuf) -> Self
    {
        let repository = gix::discover(&repo_path).ok();
        if let Some(ref repo) = repository {
            info!("Git repository detected at: {:?}", repo.path());
        } else {
            warn!("No git repository found starting from: {:?}", repo_path);
        }
        Self {
            repository,
            tracked_paths: Vec::new(),
            statuses: HashMap::new(),
            repo_path,
        }
    }

    /// Refresh the internal state synchronously
    fn refresh_paths(&mut self, file_paths: Vec<PathBuf>)
    {
        self.tracked_paths = file_paths;
        self.statuses.clear();
        if let Some(repository) = &self.repository {
            self.statuses = Self::get_scm_statuses_for_files(repository, &self.tracked_paths);
        }
    }

    /// Re-discover repository and refresh state
    fn refresh_repository(&mut self)
    {
        self.repository = gix::discover(&self.repo_path).ok();
        self.refresh_paths(self.tracked_paths.clone());
    }

    /// Get SCM statuses for a batch of files
    fn get_scm_statuses_for_files(
        repository: &Repository,
        file_paths: &[PathBuf],
    ) -> HashMap<String, CachedStatus>
    {
        let mut statuses = HashMap::new();
        for file_path in file_paths {
            let status = {
                if !file_path.exists() {
                    ScmStatus::DoesNotExist
                } else {
                    let repo_path = repository.path().parent().unwrap_or(repository.path());
                    let relative_path = match file_path.strip_prefix(repo_path) {
                        Ok(path) => path,
                        Err(_) => {
                            // File is not in repository
                            let path_key = file_path.to_string_lossy().to_string();
                            statuses.insert(path_key, CachedStatus::new(ScmStatus::Unknown));
                            continue;
                        }
                    };
                    let path_pattern = relative_path.to_string_lossy();
                    let path_pattern = BStr::new(path_pattern.as_bytes());
                    let status_result = repository.status(gix::progress::Discard);
                    let status = match status_result {
                        Ok(status) => status,
                        Err(_) => {
                            // If status check fails, mark as unmodified
                            let path_key = file_path.to_string_lossy().to_string();
                            statuses.insert(path_key, CachedStatus::new(ScmStatus::Unmodified));
                            continue;
                        }
                    };
                    let iter_result = status.into_iter([path_pattern.to_owned()]);
                    let mut iter = match iter_result {
                        Ok(iter) => iter,
                        Err(_) => {
                            let path_key = file_path.to_string_lossy().to_string();
                            statuses.insert(path_key, CachedStatus::new(ScmStatus::Unmodified));
                            continue;
                        }
                    };
                    let mut found = ScmStatus::Unmodified;
                    while let Some(Ok(item)) = iter.next() {
                        match item {
                            Item::TreeIndex(Change::Addition { .. }) => {
                                found = ScmStatus::New;
                                break;
                            }
                            Item::TreeIndex(Change::Modification { .. }) => {
                                found = ScmStatus::Modified;
                                break;
                            }
                            Item::IndexWorktree(index_worktree::Item::Modification { .. }) => {
                                found = ScmStatus::Modified;
                                break;
                            }
                            Item::IndexWorktree(index_worktree::Item::Rewrite { .. }) => {
                                found = ScmStatus::New;
                                break;
                            }
                            Item::IndexWorktree(index_worktree::Item::DirectoryContents {
                                ..
                            }) => {
                                found = ScmStatus::New;
                                break;
                            }
                            _ => (),
                        }
                    }
                    found
                }
            };
            let path_key = file_path.to_string_lossy().to_string();
            statuses.insert(path_key, CachedStatus::new(status));
        }

        statuses
    }
}
