<h1 align="center">git locate</h1>

`git locate` lists your branches (from newest to oldest) and you choose one.
If the branch is already checked out in a worktree, it prints the path of that
worktree.  Otherwise, it checks out the branch in the current worktree.  You use
it like this:

```console
$ cd $(git locate)
```
