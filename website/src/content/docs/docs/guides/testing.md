---
title: Testing archives
description: Check that an archive is intact before you rely on it.
---

Testing reads every file in the archive and checks it against the checksum stored inside
(7-Zip’s `t` command), without writing anything to disk. It catches truncated downloads, damaged
disks and wrong passwords.

- In an open archive, click **Test**.
- The job runs in [Activity](/docs/guides/activity/) with its progress.
- When it ends, **Everything is OK** or the list of files 7-Zip reported as broken.

For an encrypted archive you are asked for the password: testing with the wrong password fails
like a damaged archive would.

:::tip
Test a new archive before deleting the originals — or before burning it, uploading it or putting it
into long-term storage.
:::

Split archives are tested as a whole: open any volume, with every part in the same folder.
