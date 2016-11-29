from plumbum import FG

from gg.gateways.git import git_cmd
from gg.lib.log import logger


def safe_git_delete_branch(branch_name: str) -> int:
    """Wrap the git_delete_branch in a try-except"""
    try:
        git_delete_branch(branch_name)
    except Exception as e:
        logger.error("Could not delete branch: %s, error: %s" % (branch_name, str(e)))

def git_delete_branch(branch_name) -> int:
    """Delete a git branch"""
    command = git_cmd['branch', '-D', branch_name]
    return (command) & FG
