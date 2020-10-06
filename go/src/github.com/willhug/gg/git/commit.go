package git

import (
	"fmt"
	"strings"
)

type Commit struct {
	Hash   string
	Title  string
	Author string
	Body   string
}

func (c Commit) ShortHash() string {
	return c.Hash[:7]
}

func (g *Git) GetCurrentCommit() (Commit, error) {
	return g.GetCommit("HEAD")
}

func (g *Git) GetCommit(ref string) (Commit, error) {
	commits, err := g.GetCommits(ref+"^", ref)
	if err != nil {
		return Commit{}, err
	}
	if len(commits) == 0 {
		return Commit{}, fmt.Errorf("could not find ref %s", ref)
	}
	return commits[0], nil
}

const (
	paramDelimiter = "~*~*~*~"
	lineDelimiter  = "^*^*^*^*^*^*^*^*^*"
)

func (g *Git) GetCommits(startRef, endRef string) ([]Commit, error) {
	str, err := g.runner.RunAndGetAllAsString(
		"log",
		"--pretty=format:%H"+paramDelimiter+"%s"+paramDelimiter+"%an"+paramDelimiter+"%b"+lineDelimiter,
		startRef+".."+endRef,
	)
	if err != nil {
		return nil, err
	}
	commitStrs := strings.Split(str, lineDelimiter)
	res := make([]Commit, len(commitStrs))
	for i, commitStr := range commitStrs {
		commitSplit := strings.Split(commitStr, paramDelimiter)
		if len(commitSplit) != 4 {
			return nil, fmt.Errorf("could not properly split param info: %s", commitStr)
		}
		res[i].Hash = commitSplit[0]
		res[i].Title = commitSplit[1]
		res[i].Author = commitSplit[2]
		res[i].Body = commitSplit[3]
	}
	return res, nil
}
