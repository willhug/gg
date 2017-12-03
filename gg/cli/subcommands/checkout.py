from plumbum import cli

from gg.actions.checkout import checkout
from gg.cli.gg import GG

@GG.subcommand("co")
class GGCheckout(cli.Application):
    DESCRIPTION = "Checkout a branch (can use the current branch for relative information)"

    getNext = cli.Flag(['-n', '--next'], help="checkout the next branch commit")
    getPrev = cli.Flag(['-p', '--prev'], help="checkout the previous branch commit")
    part = cli.SwitchAttr(['-a', '--part'], float, help="part number to check out", default=None)
    feature_name = cli.SwitchAttr(['-f', '--feature'], str, help="base branch name to check out", default=None)

    def main(self, *args):
        direction = None
        if self.getNext:
            direction = True
        elif self.getPrev:
            direction = False

        return checkout(
            direction=direction,
            part=self.part,
            feature=self.feature_name,
        )

