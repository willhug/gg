from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git import REPO_USER, REPO_NAME, IS_GITHUB
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.push import git_push
from gg.gateways.github.pr_info import get_core_pulL_request
from gg.gateways.github.pull_request import create_pull_request
from gg.lib.branch_name import get_prefix_branch_name
from gg.lib.log import logger


@GG.subcommand("pr")
class GGPullRequest(cli.Application):
    DESCRIPTION = "create a pull request for the branch, or update the pull request"

    def main(self, *args):
        if self.is_github():
            return self.create_github_pr()
        logger.error('no support for arc yet')
        return 1

    def is_github(self):
        return IS_GITHUB

    def create_github_pr(self) -> int:
        current_branch = get_current_branch()
        if not current_branch:
            print('ERROR no branch currently checked out, cannot checkout relative commit')
            return 1

        self.push_to_remote(current_branch)

        if self.has_existing_pr(current_branch):
            # Done the push was all we needed to do
            return 0

        self.create_pull_request(current_branch)

    def push_to_remote(self, branch: str):
        """Push the branch and it's 'prefix' branch to the remote"""
        # Push the prefix
        prefix_branch = get_prefix_branch_name(branch)
        git_checkout(prefix_branch)
        git_push(prefix_branch, force=True)

        # Push the main branch
        git_checkout(branch)
        git_push(branch, force=True)

    def has_existing_pr(self, branch_name: str) -> bool:
        """Return true of there's already a PR for this remote branch"""
        try:
            get_core_pulL_request(branch_name, REPO_USER, REPO_NAME)
        except Exception:
            logger.info("PR doesn't exist")
            return False

        logger.info("PR already exists")
        return True

    def create_pull_request(self, branch: str):
        """Create a pull request"""
        prefix_branch = get_prefix_branch_name(branch)

        git_checkout(branch)
        create_pull_request(base_branch=prefix_branch)
