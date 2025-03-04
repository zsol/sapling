# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This software may be used and distributed according to the terms of the
# GNU General Public License version 2.

# Extension dedicated to test patch.diff() upgrade modes

from __future__ import absolute_import

from edenscm import error, patch, registrar, scmutil


cmdtable = {}
command = registrar.command(cmdtable)


@command(
    "autodiff",
    [("", "git", "", "git upgrade mode (yes/no/auto/warn/abort)")],
    "[OPTION]... [FILE]...",
)
def autodiff(ui, repo, *pats, **opts):
    diffopts = patch.difffeatureopts(ui, opts)
    git = opts.get("git", "no")
    brokenfiles = set()
    losedatafn = None
    if git in ("yes", "no"):
        diffopts.git = git == "yes"
        diffopts.upgrade = False
    elif git == "auto":
        diffopts.git = False
        diffopts.upgrade = True
    elif git == "warn":
        diffopts.git = False
        diffopts.upgrade = True

        def losedatafn(fn=None, **kwargs):
            brokenfiles.add(fn)
            return True

    elif git == "abort":
        diffopts.git = False
        diffopts.upgrade = True

        def losedatafn(fn=None, **kwargs):
            raise error.Abort("losing data for %s" % fn)

    else:
        raise error.Abort("--git must be yes, no or auto")

    node1, node2 = scmutil.revpair(repo, [])
    m = scmutil.match(repo[node2], pats, opts)
    it = patch.diff(
        repo, repo[node1], repo[node2], match=m, opts=diffopts, losedatafn=losedatafn
    )
    for chunk in it:
        ui.writebytes(chunk)
    for fn in sorted(brokenfiles):
        ui.write(("data lost for: %s\n" % fn))
