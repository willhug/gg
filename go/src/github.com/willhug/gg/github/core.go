package github

import (
	"context"

	"github.com/google/go-github/v32/github"
	"github.com/willhug/gg/config"
	"github.com/willhug/gg/git"
	"golang.org/x/oauth2"
)

type Github struct {
	client *github.Client
	gitCfg *git.CurrentConfig
}

func NewGithub(c *config.Config, currentConfig *git.CurrentConfig) *Github {
	ctx := context.Background()
	ts := oauth2.StaticTokenSource(
		&oauth2.Token{AccessToken: "... your access token ..."},
	)
	tc := oauth2.NewClient(ctx, ts)

	client := github.NewClient(tc)
	return &Github{
		client: client,
		gitCfg: currentConfig,
	}
}

func (g *Github) NewPR() {
	// TODO Open the editor and open it.
	//g.client.PullRequests.Create()
}
