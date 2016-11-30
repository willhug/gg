from plumbum import FG

from gg.gateways.git import git_cmd
from gg.lib.log import logger


def safe_git_delete_remote_branch(remote: str, branch_name: str) -> int:
    """Wrap the git_delete_branch in a try-except"""
    try:
        git_delete_remote_branch(remote, branch_name)
    except Exception as e:
        logger.error("Could not delete remote branch: %s, error: %s" % (branch_name, str(e)))

def git_delete_remote_branch(remote: str, branch_name: str) -> int:
    """Delete a git branch"""
    command = git_cmd['push', remote, '--delete', branch_name]
    return (command) & FG
