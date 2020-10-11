package config

import (
	"encoding/json"
	"io/ioutil"
	"os"

	"github.com/samsarahq/go/oops"
)

const GGConfigFile = "~/.ggconfig"

type Config struct {
	GithubAuthToken string `json:"github_auth_token"`
}

func ReadConfig() (*Config, error) {
	file, err := os.Open(GGConfigFile)
	if err != nil {
		return nil, oops.Wrapf(err, "could not open config")
	}
	b, err := ioutil.ReadAll(file)
	if err != nil {
		return nil, oops.Wrapf(err, "could not read file")
	}
	var c Config
	if err = json.Unmarshal(b, &c); err != nil {
		return nil, oops.Wrapf(err, "could not unmarshal")
	}
	return &c, nil
}
