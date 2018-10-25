from __future__ import absolute_import

from os import sys, path
sys.path.append(path.join(path.dirname(__file__), '..'))

from gg.cli.subcommands.checkout import GGCheckout #nolint
from gg.cli.subcommands.delete import GGDelete #nolint
from gg.cli.subcommands.diff import GGDiff #nolint
from gg.cli.subcommands.fixup import GGFixup #nolint
from gg.cli.subcommands.land import GGLand #nolint
from gg.cli.subcommands.landrebase import GGLandRebase #nolint
from gg.cli.subcommands.log import GGLog #nolint
from gg.cli.subcommands.new import GGNew #nolint
from gg.cli.subcommands.pullrequest import GGPullRequest #nolint
from gg.cli.subcommands.push import GGPush #nolint
from gg.cli.subcommands.rebase import GGRebase #nolint
from gg.cli.subcommands.rebaseall import GGRebaseAll #nolint
from gg.cli.subcommands.renumber import GGRenumber #nolint
from gg.cli.subcommands.review import GGReview #nolint
from gg.cli.subcommands.updatebase import GGUpdateBase #nolint
from gg.cli.gg import GG


def run():
    GG.run()

if __name__ == "__main__":
    run()
