package git

func (g *Git) CherryPick(startRef, endRef string) error {
	return g.runner.Run("cherry-pick", startRef+".."+endRef)
}

func (g *Git) CherryPickWithStrategy(startRef, endRef, strategy string) error {
	return g.runner.Run("cherry-pick", startRef+".."+endRef, "--strategy-option", strategy)
}

func (g *Git) CherryPickContinue() error {
	return g.runner.Run("cherry-pick", "--continue")
}

func (g *Git) CherryPickAbort() error {
	return g.runner.Run("cherry-pick", "--abort")
}
