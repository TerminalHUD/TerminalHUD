# Source organization

Organize responsibilities within the existing architecture and crates. Give
each function one concern; composition functions call named operations. Keep
crate roots and module facades short, use private implementation modules, and
reexport the intended public interface explicitly.

A file of roughly one readable screen (around 100 lines or fewer, usually
smaller) is a review prompt, not a hard CI gate. Keep a cohesive owning state
private and complete rather than splitting it to meet a line count. Local
helpers belong beside their operation; genuinely shared helpers use concrete
names that explain their responsibility. Shallow folders should provide useful
context. Avoid generic utility dumps, arbitrary slices of a large file, and new
traits or crates created only to move code. Mirror responsibilities in tests.

During structural cleanup, preserve APIs, errors, ordering, platform cfgs,
ownership and Drop behavior, and borrow/lock/callback timing. Separate structural
moves from behavior fixes; where practical, checkpoint working bodies before
moving them. Review both the result and the preservation evidence.
