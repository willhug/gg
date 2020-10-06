package git

func (g *Git) DiffCur(ref string) error {
	return g.Diff(ref, "HEAD")
}

func (g *Git) Diff(startRef, endRef string) error {
	return g.runner.Run("diff", startRef+".."+endRef)
}
