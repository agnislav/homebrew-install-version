Install a particular formula version with Homebrew.
=

This is a simple app to install a particular version of a formula with Homebrew. It's useful when you need to install a specific version of a formula, but the version is not available in the tap.

It's simple, it may be buggy, and it may not work for all formulas. But it works for me, and I hope it works for you too.

Usage
=
homebrew_install_version &lt;formula&gt; &lt;version&gt;

Known issues:
=
- it checks only the last 100 commits of a formula
  - it may not work for older versions
- it doesn't check whether the version is skipped in the formula
  - e.g., if the formula has versions 1.4, 1.3, 1.0, and you'll try to install 1.2, it will fail
- the output is ugly
- most probably it's not optimised and use some bad practices
  - don't beat me hard, it's my first Rust app

The history behind
=
I'm sick with cocoapods issues. So, I needed a tool to install a previous version of a package or one before. 