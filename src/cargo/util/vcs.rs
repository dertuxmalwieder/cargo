use crate::util::CargoResult;
use cargo_util::paths;
use cargo_util::ProcessBuilder;
use std::path::Path;
use url::Url;

// Check if we are in an existing repo. We define that to be true if either:
//
// 1. We are in a git repo and the path to the new package is not an ignored
//    path in that repo.
// 2. We are in an HG repo.
pub fn existing_vcs_repo(path: &Path, cwd: &Path) -> bool {
    fn in_git_repo(path: &Path, cwd: &Path) -> bool {
        if let Ok(repo) = GitRepo::discover(path, cwd) {
            // Don't check if the working directory itself is ignored.
            if repo.workdir().map_or(false, |workdir| workdir == path) {
                true
            } else {
                !repo.is_path_ignored(path).unwrap_or(false)
            }
        } else {
            false
        }
    }

    in_git_repo(path, cwd) || HgRepo::discover(path, cwd).is_ok()
}

pub struct HgRepo;
pub struct GitRepo;
pub struct PijulRepo;
pub struct FossilRepo;
pub struct SvnRepo;

impl GitRepo {
    pub fn init(path: &Path, _: &Path) -> CargoResult<GitRepo> {
        git2::Repository::init(path)?;
        Ok(GitRepo)
    }
    pub fn discover(path: &Path, _: &Path) -> Result<git2::Repository, git2::Error> {
        git2::Repository::discover(path)
    }
}

impl HgRepo {
    pub fn init(path: &Path, cwd: &Path) -> CargoResult<HgRepo> {
        ProcessBuilder::new("hg")
            .cwd(cwd)
            .arg("init")
            .arg(path)
            .exec()?;
        Ok(HgRepo)
    }
    pub fn discover(path: &Path, cwd: &Path) -> CargoResult<HgRepo> {
        ProcessBuilder::new("hg")
            .cwd(cwd)
            .arg("--cwd")
            .arg(path)
            .arg("root")
            .exec_with_output()?;
        Ok(HgRepo)
    }
}

impl PijulRepo {
    pub fn init(path: &Path, cwd: &Path) -> CargoResult<PijulRepo> {
        ProcessBuilder::new("pijul")
            .cwd(cwd)
            .arg("init")
            .arg(path)
            .exec()?;
        Ok(PijulRepo)
    }
}

impl FossilRepo {
    pub fn init(path: &Path, cwd: &Path) -> CargoResult<FossilRepo> {
        // fossil doesn't create the directory so we'll do that first
        paths::create_dir_all(path)?;

        // set up the paths we'll use
        let db_fname = ".fossil";
        let mut db_path = path.to_owned();
        db_path.push(db_fname);

        // then create the fossil DB in that location
        ProcessBuilder::new("fossil")
            .cwd(cwd)
            .arg("init")
            .arg(&db_path)
            .exec()?;

        // open it in that new directory
        ProcessBuilder::new("fossil")
            .cwd(&path)
            .arg("open")
            .arg(db_fname)
            .exec()?;

        Ok(FossilRepo)
    }
}

impl SvnRepo {
    pub fn init(path: &Path, cwd: &Path) -> CargoResult<SvnRepo> {
        // SVN doesn't create the directory so we'll do that first.
        paths::create_dir_all(path)?;

        // Create a repository ...
        ProcessBuilder::new("svnadmin")
            .cwd(cwd)
            .arg("create")
            .arg(path)
            .exec()?;
        
        // ... and check it out:
        let filepath = Url::from_file_path(path).unwrap();
        ProcessBuilder::new("svn")
            .cwd(cwd)
            .arg("checkout")
            .arg(filepath.as_str())
            .exec()?;
        Ok(SvnRepo)
    }
}
