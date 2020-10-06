package git

func (g *Git) RebaseInteractive(ref string) error {
	return g.runner.Run("rebase", "-i", ref)
}

func (g *Git) RebaseAbort() error {
	return g.runner.Run("rebase", "--abort")
}

func (g *Git) RebaseContinue() error {
	return g.runner.Run("rebase", "--continue")
}

// Fix the commit times (can cause github to show the commits out of order)
func (g *Git) RebaseFixCommitTimes(ref string) error {
	return g.runner.Run("rebase", ref, "--exec", "'git commit --amend --reset-author --no-edit'")
}
