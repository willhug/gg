package git

import "strings"

type CurrentConfig struct {
	IsGithub  bool
	OriginUrl string
	RepoName  string
	RepoUser  string
}

func Config(g *Git) (*CurrentConfig, error) {
	return g.GetConfig()
}

func (g *Git) GetConfig() (*CurrentConfig, error) {
	originUrl, err := g.runner.RunWithSingleResp("config", "remote.origin.url")
	if err != nil {
		return nil, err
	}
	cfg := &CurrentConfig{
		OriginUrl: originUrl,
	}
	originUrl = strings.TrimSuffix(originUrl, "ssh://")
	osplit := strings.Split(originUrl, ":")
	if strings.Contains(osplit[0], "github.com") {
		cfg.IsGithub = true
	}
	path := osplit[1]
	pathSplit := strings.Split(path, "/")
	cfg.RepoUser = pathSplit[0]
	repoName := pathSplit[1]
	repoName = strings.TrimSuffix(repoName, ".git")
	cfg.RepoName = repoName
	return cfg, nil
}
