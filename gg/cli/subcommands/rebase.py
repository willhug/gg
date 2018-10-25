from plumbum import cli, colors

from gg.cli.gg import GG
from gg.actions.rebase import abort_rebase, continue_rebase, start_rebase


@GG.subcommand("rebase")
class GGRebase(cli.Application):
    DESCRIPTION = """
Rebase onto the next branch in the stack
"""
    onto = cli.SwitchAttr(['-o', '--onto'], str, help="git ref to rebase onto, defaults to the previous branch in the stack", default=None)
    strategy = cli.SwitchAttr(['-s', '--strategy'], str, help="git strategy-option for cherry-pick", default=None)

    rebase_abort = cli.Flag(['-a', '--abort'], help="abort the rebase")
    rebase_continue = cli.Flag(['-c', '--continue'], help="continue the rebase")

    def main(self, *args):
        if self.rebase_abort:
            return abort_rebase()
        if self.rebase_continue:
            return continue_rebase()

        start_rebase(self.onto, strategy=self.strategy)
