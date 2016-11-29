from plumbum import FG

from gg.gateways.git import git_cmd

def git_checkout(revision_identifier) -> int:
    """Checkout the revision identifier that is passed in"""
    command = git_cmd['checkout', revision_identifier]
    return (command) & FG
