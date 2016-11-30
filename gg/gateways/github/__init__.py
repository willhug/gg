import os

GITHUB_API_URL = "https://api.github.com"
GITHUB_TOKEN_ENVVAR = "GITHUB_TOKEN"
GITHUB_TOKEN = os.environ.get(GITHUB_TOKEN_ENVVAR)
