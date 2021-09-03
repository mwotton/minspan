# minspan

This is a tiny crate to find the minimal bounds of a string within another one.
The needle must be found in its entirety within the haystack, but there may be
any number of intervening characters that are just chaff. This is useful for
applications like fuzzy-completers, where a shorter match is generally preferable
("curl" matches "curl https://rust-lang.org" better than it matches "colossally
urban lapidarians").

The interface is small but under flux, it's possible that a slice is a better
return value than bare integer indices.
