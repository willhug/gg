from plumbum import local

git_cmd = local['git']

try: 
    ORIGIN_URL = git_cmd['config', 'remote.origin.url']()
except:
    ORIGIN_URL = "test@not.com:/test.git"

IS_GITHUB = "github.com" in ORIGIN_URL

if ORIGIN_URL.startswith("ssh://git@github.com/"):
    path = ORIGIN_URL[21:]
else:
    path = ORIGIN_URL.split(":")[-1]

REPO_USER = path.split("/")[0].strip()
repo = path.split("/")[1].strip()

if repo.endswith(".git"):
    REPO_NAME = repo[:-4]
else:
    REPO_NAME = repo
