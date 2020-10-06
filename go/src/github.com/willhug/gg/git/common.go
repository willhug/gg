package git

import (
	"strings"

	"github.com/willhug/gg/cli"
)

// Git is a struct for executing a CLI Git command
// TODO use https://github.com/go-git/go-git
type Git struct {
	runner *cli.Runner
}

func New() *Git {
	return &Git{
		runner: cli.NewRunner("git"),
	}
}

func (g *Git) FetchMaster() error {
	return g.runner.Run("fetch", "-p", "origin", "master")
}

func (g *Git) ForcePush(revisions ...string) error {
	return g.pushMulti(revisions, true)
}

func (g *Git) Push(revisions ...string) error {
	return g.pushMulti(revisions, false)
}

func (g *Git) pushMulti(revisions []string, force bool) error {
	args := []string{}
	if force {
		args = append(args, "-f")
	}
	args = append(args, "origin")
	args = append(args, revisions...)
	return g.runner.Run("push", args...)
}

func (g *Git) Checkout(revision string) error {
	return g.runner.Run("checkout", revision)
}

func (g *Git) NewCheckout(branch string) error {
	return g.runner.Run("checkout", "-b", branch)
}

func (g *Git) Delete(branch string) {
	g.runner.SoftRun("branch", "-D", branch)
}

func (g *Git) DeleteRemote(remote, branch string) {
	g.runner.SoftRun("push", remote, "--delete", branch)
}

func (g *Git) CurrentBranch() (string, error) {
	return g.runner.RunWithSingleResp("rev-parse", "--abbrev-ref", "HEAD")
}

func (g *Git) AllBranches() ([]string, error) {
	lines, err := g.runner.RunAndGetLines("branch", "--format", "'%(refname:short)'")
	if err != nil {
		return nil, err
	}
	if len(lines) < 1 {
		return lines, nil
	}
	if strings.HasPrefix(lines[0], "(HEAD detached at") {
		lines = lines[1:]
	}
	return lines, nil
}

func (g *Git) ResetHard(branch string) error {
	return g.reset(branch, true)
}
func (g *Git) Reset(branch string) error {
	return g.reset(branch, false)
}

func (g *Git) reset(branch string, hard bool) error {
	args := []string{}
	if hard {
		args = append(args, "--hard")
	}
	args = append(args, branch)
	return g.runner.Run("reset", args...)
}
