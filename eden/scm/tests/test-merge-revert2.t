#debugruntest-compatible
# Copyright (c) Meta Platforms, Inc. and affiliates.
# Copyright (c) Mercurial Contributors.
#
# This software may be used and distributed according to the terms of the
# GNU General Public License version 2 or any later version.

  $ setconfig devel.segmented-changelog-rev-compat=true
  $ hg init repo
  $ cd repo

  $ echo 'added file1' > file1
  $ echo 'another line of text' >> file1
  $ echo 'added file2' > file2
  $ hg add file1 file2
  $ hg commit -m 'added file1 and file2'

  $ echo 'changed file1' >> file1
  $ hg commit -m 'changed file1'

  $ hg -q log
  dfab7f3c2efb
  c3fa057dd86f
  $ hg id
  dfab7f3c2efb

  $ hg goto -C 0
  1 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ hg id
  c3fa057dd86f

  $ echo 'changed file1' >> file1
  $ hg id
  c3fa057dd86f+

  $ hg revert --no-backup --all
  reverting file1
  $ hg diff
  $ hg status
  $ hg id
  c3fa057dd86f

  $ hg goto
  1 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ hg diff
  $ hg status
  $ hg id
  dfab7f3c2efb

  $ hg goto -C 0
  1 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ echo 'changed file1 different' >> file1

  $ hg goto
  merging file1
  warning: 1 conflicts while merging file1! (edit, then use 'hg resolve --mark')
  0 files updated, 0 files merged, 0 files removed, 1 files unresolved
  use 'hg resolve' to retry unresolved file merges
  [1]

  $ hg diff --nodates
  diff -r dfab7f3c2efb file1
  --- a/file1
  +++ b/file1
  @@ -1,3 +1,7 @@
   added file1
   another line of text
  +<<<<<<< working copy: c3fa057dd86f - test: added file1 and file2
  +changed file1 different
  +=======
   changed file1
  +>>>>>>> destination:  dfab7f3c2efb - test: changed file1

  $ hg status
  M file1
  ? file1.orig
  $ hg id
  dfab7f3c2efb+

  $ hg revert --no-backup --all
  reverting file1
  $ hg diff
  $ hg status
  ? file1.orig
  $ hg id
  dfab7f3c2efb

  $ hg revert -r tip --no-backup --all
  $ hg diff
  $ hg status
  ? file1.orig
  $ hg id
  dfab7f3c2efb

  $ hg goto -C
  0 files updated, 0 files merged, 0 files removed, 0 files unresolved
  $ hg diff
  $ hg status
  ? file1.orig
  $ hg id
  dfab7f3c2efb
