<h1 align="center"> Ripcord </h1>

> This project is in a highly experimental stage therefore breaking changes will come as development continues. This notice will remain until version 1.0.0 (or specified otherwise) which will mark the first stabilized release.

<strong> Ripcord is a batteries-includes text processor serving as the backbone of text editors and applications alike. </strong>

Prelude
=

The standards for the traditional text editor have come a long way with their respective capabilities and quirks. There is one key design goal amongst them all - performance. Ripcord started out as a by-product from an exploratory dive into the science & makings of text editors, but has since become a crucial component for certain products I have in the making.

Modus Operandi
=

The main intent is to provide a, subjectively minimal, lightweight and performant library for creating processors usable as a text editor backend. Aspects of design include, but not limited to:

- $log(n)$ time for operations such as insertions and deletions.