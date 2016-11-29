from plumbum import cli, colors

from gg.cli.gg import GG
from gg.gateways.git.branch_checkout import git_checkout
from gg.gateways.git.branch_delete import safe_git_delete_branch
from gg.gateways.git.branch_info import get_current_branch
from gg.gateways.git.branch_create import git_checkout_new_branch
from gg.gateways.git.cherry_pick import git_cherry_pick, git_cherry_pick_abort, git_cherry_pick_continue
from gg.gateways.git.reset import git_reset
from gg.lib.branch_name import get_prefix_branch_name, get_previous_branch
from gg.lib.log import logger

TMP_PREFIX = "_tmp_-"


@GG.subcommand("rebase")
class GGRebase(cli.Application):
    DESCRIPTION = """
Rebase onto the next branch in the stack
"""
    onto = cli.SwitchAttr(['-o', '--onto'], str, help="git ref to rebase onto, defaults to the previous branch in the stack", default=None)

    rebase_abort = cli.Flag(['-a', '--abort'], help="abort the rebase")
    rebase_continue = cli.Flag(['-c', '--continue'], help="continue the rebase")
    ignore_errors = cli.Flag(['--ignore-errors'], help="ignore the errors from rebase abort or rebase continue")

    def main(self, *args):
        if self.rebase_abort:
            return self.abort_rebase()
        if self.rebase_continue:
            return self.continue_rebase()

        self.start_rebase()

    def abort_rebase(self):
        tmp_branch = get_current_branch()
        branch = self.get_branch_from_tmp(tmp_branch)
        if branch is None:
            logger.error("Not on a tmp branch, can't abort rebase")
            return 1

        # Stop the rebase
        try:
            git_cherry_pick_abort()
        except Exception as e:
            if not self.ignore_errors:
                raise e

        # Checkout old branch
        git_checkout(branch)

        # Delete the tmp branch
        safe_git_delete_branch(tmp_branch)

        return 0

    def continue_rebase(self):
        tmp_branch = get_current_branch()
        branch = self.get_branch_from_tmp(tmp_branch)
        if branch is None:
            logger.error("Not on a tmp branch, can't continue rebase")
            return 1

        # continue the rebase
        try:
            git_cherry_pick_continue()
        except Exception as e:
            if not self.ignore_errors:
                raise e

        rebase_onto_branch = get_previous_branch(branch)

        self.finish_rebase(
            tmp_branch,
            to_rebase_branch=branch,
            rebase_onto_branch=rebase_onto_branch,
        )


    def start_rebase(self):
        current_branch = get_current_branch()
        if current_branch is None:
            logger.error("No branch checked out")
            return 1

        if self.onto is not None:
            return self.rebase_onto(to_rebase_branch=current_branch, rebase_onto_branch=self.onto)

        previous_branch = get_previous_branch(current_branch)
        if previous_branch is None:
            logger.error("Could not determine branch to rebase onto")
            return 1

        return self.rebase_onto(to_rebase_branch=current_branch, rebase_onto_branch=previous_branch)

    def rebase_onto(self, to_rebase_branch: str, rebase_onto_branch: str) -> int:
        """Rebase (we'll actually use a cherry-pick) a branch onto another one"""
        tmp_branch = "_tmp_-" + to_rebase_branch

        # Create a new tmp branch in "onto"
        git_checkout(rebase_onto_branch)
        git_checkout_new_branch(tmp_branch)

        # Cherry-pick the changes
        prefix_branch = get_prefix_branch_name(to_rebase_branch)
        git_cherry_pick(start_ref=prefix_branch, end_ref=to_rebase_branch)

        self.finish_rebase(
            tmp_branch,
            to_rebase_branch,
            rebase_onto_branch
        )

    def finish_rebase(
        self,
        tmp_branch: str,
        to_rebase_branch: str,
        rebase_onto_branch: str,
    ):
        """Finish/cleanup the rebase after the cherry-pick"""
        prefix_branch = get_prefix_branch_name(to_rebase_branch)

        # Reset the prefix and to_rebase branches
        git_checkout(prefix_branch)
        git_reset(new_ref=rebase_onto_branch, hard=True)

        git_checkout(to_rebase_branch)
        git_reset(new_ref=tmp_branch, hard=True)

        # Cleanup the tmp branch
        safe_git_delete_branch(tmp_branch)

    def get_tmp_branch_name(self, branch_name: str) -> str:
        return TMP_PREFIX + branch_name

    def get_branch_from_tmp(self, tmp_branch_name: str) -> str:
        if tmp_branch_name.startswith(TMP_PREFIX):
            return tmp_branch_name[len(TMP_PREFIX):]

        logger.error("Not in a rebase, no temporary branch checked out")
        return None
