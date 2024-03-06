mod fuzzy;

use anyhow::ensure;
use gix::{date::Time, refs::FullName, Repository};
use std::{collections::HashMap, fmt, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let repo = gix::discover(".")?;
    let mut branches = branches(&repo)?;
    branches.sort_by_key(|x| std::cmp::Reverse(x.committer_time));

    let selection = crate::fuzzy::run(branches)?;
    if let Some(selection) = selection {
        if let Some(worktree) = &selection.worktree {
            println!("{}", worktree.display());
        } else {
            let status = std::process::Command::new("git")
                .args(["switch", &selection.name.shorten().to_string()])
                .status()?;
            ensure!(status.success());
            println!(".");
        }
    } else {
        println!(".");
    }

    Ok(())
}

#[derive(Clone)]
struct Branch {
    name: FullName,
    worktree: Option<PathBuf>,
    committer_time: Time,
}
fn branches(repo: &Repository) -> anyhow::Result<Vec<Branch>> {
    let mut worktrees: HashMap<FullName, PathBuf> = HashMap::default();
    {
        let main_repo = repo.main_repo()?;
        if let Some((head, dir)) = main_repo.head_name()?.zip(main_repo.work_dir()) {
            worktrees.insert(head, dir.to_path_buf());
        }
    }
    for worktree in repo.worktrees()? {
        let dir = worktree.base()?;
        if let Some(head) = worktree.into_repo()?.head_name()? {
            worktrees.insert(head, dir);
        }
    }
    let mut branches = vec![];
    for branch in repo.references()?.local_branches()?.peeled() {
        let branch = branch.unwrap();
        let worktree = worktrees.remove(branch.name());
        let committer_time = branch.id().object()?.try_into_commit()?.time()?;
        branches.push(Branch {
            name: branch.name().to_owned(),
            worktree,
            committer_time,
        });
    }
    Ok(branches)
}
impl fmt::Display for Branch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.shorten();
        write!(f, "{}", name)?;
        if let Some(worktree) = &self.worktree {
            write!(f, "{:<w$}", " ", w = 40 - name.len())?;
            write!(f, "[{}]", worktree.display())?;
        }
        Ok(())
    }
}
