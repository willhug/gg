from plumbum import cli, colors

from gg.actions.land import has_existing_pr
from gg.cli.gg import GG
from gg.actions.rebase import abort_rebase, continue_rebase, start_rebase
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.push import git_push
from gg.lib.branch_name import get_previous_branch, get_next_branch


@GG.subcommand("rebaseall")
class GGRebaseAll(cli.Application):
    DESCRIPTION = """
Rebase all the stacked commits
"""
    onto = cli.SwitchAttr(['-o', '--onto'], str, help="git ref to rebase onto, defaults to the previous branch in the stack", default=None)
    strategy = cli.SwitchAttr(['-s', '--strategy'], str, help="git strategy-option for cherry-pick", default=None)

    push = cli.Flag(['-p', '--push'], help="push changes to remote (if applicable)")

    rebase_abort = cli.Flag(['-a', '--abort'], help="abort the rebase")
    rebase_continue = cli.Flag(['-c', '--continue'], help="continue the rebase")

    def main(self, *args):
        if self.rebase_abort:
            return abort_rebase()
        if self.rebase_continue:
            if continue_rebase():
                return 1
            self.loop_rebase(get_next_branch(get_current_branch()))
            return

        self.loop_rebase(get_current_branch())

    def loop_rebase(self, start_branch: str):
        if start_branch is None:
            return

        current_branch = start_branch

        git_checkout(current_branch)

        onto_branch = get_previous_branch(current_branch)
        if onto_branch is None:
            onto_branch = self.onto

        while current_branch is not None:
            print("Rebasing " + current_branch + " onto " + onto_branch)
            git_checkout(current_branch)
            start_rebase(onto_branch, strategy=self.strategy)
            if self.push and has_existing_pr(current_branch):
                git_push(current_branch, force=True)
            onto_branch = current_branch
            current_branch = get_next_branch(current_branch)
