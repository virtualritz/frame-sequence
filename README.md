# `frame-sequence-parser`

A simple parser for frame sequences strings.

This will parse [`str`] describing a sequence of frames into
a [`Vec`]`<`[`isize`]`>` containing individual frame numbers.

Mainly intended/useful for rendering/animation applications.

## Example Frame Sequence Strings

Individual frames:

`1,2,3,5,8,13` ⟶ `[1, 2, 3, 5, 8, 13]`

A sequence:

`10-15` ⟶ `[10, 11, 12, 13, 14, 15]`

With step size:

`10-20@2` ⟶ `[10, 12, 14, 16, 18, 20]`

Step size must be always positive.

To get a sequence backwards specify the range in reverse:

`42-33@3` ⟶ `[42, 39, 36, 33]`

With binary splitting:

`10-20@b` ⟶ `[10, 20, 15, 12, 17, 11, 13, 16, 18, 14, 19]`

The last frame of a sequence will be omitted if
the specified step size does not touch it:

`80-70@4` ⟶ `[80, 76, 72]`
