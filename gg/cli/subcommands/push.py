from plumbum import cli

from gg.actions.land import has_existing_pr
from gg.cli.gg import GG
from gg.gateways.git import REPO_USER, REPO_NAME, IS_GITHUB
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.push import git_push, git_push_multi
from gg.gateways.github.pr_info import get_core_pull_request
from gg.gateways.github.pull_request import create_pull_request
from gg.lib.branch_name import get_previous_branch, get_prefix_branch_name
from gg.lib.log import logger


@GG.subcommand("push")
class GGPush(cli.Application):
    DESCRIPTION = "Push the current branch to remote"
    
    start = cli.Flag(['-s', '--start'], help="push the start branch as well")

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

        if has_existing_pr(current_branch):
            git_checkout(current_branch)
            if self.start:
                git_push_multi([current_branch, get_prefix_branch_name(current_branch)], force=True)
            else:
                git_push(current_branch, force=True)
        return
