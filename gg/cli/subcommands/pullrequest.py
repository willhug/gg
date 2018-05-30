from plumbum import cli

from gg.cli.gg import GG
from gg.gateways.git import REPO_USER, REPO_NAME, IS_GITHUB
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.push import git_push
from gg.gateways.github.pr_info import get_core_pull_request
from gg.gateways.github.pull_request import create_pull_request
from gg.lib.branch_name import get_previous_branch, get_prefix_branch_name
from gg.lib.log import logger


@GG.subcommand("pr")
class GGPullRequest(cli.Application):
    DESCRIPTION = "create a pull request for the branch, or update the pull request"
    
    base = cli.SwitchAttr(['-b', '--base'], str, help="base branch name we want to merge into", default=None)
    use_previous = cli.Flag(['-p', '--onto-previous'], help="base branch will be the previous branch (not the start)")

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

        if self.base:
            return self.pushOnBase(current_branch, self.base)

        if self.use_previous:
            return self.pushOnPrevious(current_branch)

        return self.pushOnStart(current_branch)

    def pushOnBase(self, current_branch, base_branch):
        # Push the main branch
        git_checkout(current_branch)
        git_push(current_branch, force=True)
        create_pull_request(base_branch=base_branch)
        return

    def pushOnPrevious(self, current_branch):
        previous_branch = get_previous_branch(current_branch)
        if not previous_branch:
            print("ERROR no previous branch found for: " + current_branch)
            return 1

        # Push the main branch
        git_checkout(current_branch)
        git_push(current_branch, force=True)

        create_pull_request(base_branch=previous_branch)
        return

    def pushOnStart(self, current_branch):
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
            get_core_pull_request(branch_name, REPO_USER, REPO_NAME)
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
