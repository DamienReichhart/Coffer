---
title: FAQ
description: Short answers to common questions.
---

## Is Coffer made by the 7-Zip team?

No. It is an independent open-source project, not affiliated with or endorsed by 7-Zip or Igor
Pavlov.

## Why do I have to install 7-Zip separately?

Coffer is a front end: it runs the real 7-Zip on your computer. Not bundling it means 7-Zip comes
from its official source, you update it independently, and Coffer never redistributes someone
else’s software. It is one command on every platform — see [Installing 7-Zip](/docs/installing-7zip/).

## Is it free? Is there telemetry?

Free and open source (MIT licence). No telemetry, no account, no ads. The only network request is
the [update check](/docs/reference/updates/), which you can turn off.

## Can it open RAR files? Create them?

It opens and extracts RAR (and RAR5) when your 7-Zip supports it. No tool except WinRAR can create
RAR archives.

## Does it replace 7-Zip File Manager on Windows?

It covers the same everyday tasks with a modern interface on every platform. It doesn’t add 7-Zip’s
Explorer context-menu entries; installing 7-Zip for Windows gives you those as well.

## Does it work offline?

Yes. Everything works offline; only the update check needs the internet.

## Which is better, 7z or ZIP?

7z compresses better and supports encrypted file names; ZIP opens on any computer without extra
software. Use ZIP to share, 7z to store.

## Can I use it on a server or in scripts?

Coffer is a desktop app. For scripts, use 7-Zip’s command line directly — Coffer’s
[Activity log](/docs/guides/activity/) shows the exact command each job ran, which is a handy
starting point.

## How do I report a bug or request a feature?

Open an issue on [GitHub](https://github.com/DamienReichhart/Coffer/issues).
