from __future__ import absolute_import

from plumbum import FG
from plumbum import local

def create_pull_request(base_branch: str):
    hub = local['hub']

    command = hub['pull-request', '-b', base_branch]

    (command) & FG
