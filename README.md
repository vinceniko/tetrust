# Tetrust

A clone of a famous block breaking game in pure Rust first implemented in ggez and then ported to quicksilver. The quicksilver implementation is more up to date and complete.

<img src="assets/preview.gif" alt="preview" width="175"/>

## Running

### Desktop

1. git clone the repo
2. Install rust
3. `cargo run -p tetrust-ggez` or `cargo run -p tetrust-quicksilver`

### Web

1. Install cargo web and run `cargo web start -p tetrust-quicksilver`
2. Navigate to output address in browser

## Controls

|KeyCode|Action|
|-|-|
|LeftArrow|Move Tetrinome Left|
|RightArrow|Move Tetrinome Right|
|UpArrow|Rotate Tetrinome Clockwise|
|Space|Instant Drop|
|Z|Rotate Tetrinome Counter Clockwise|
|X|Rotate Tetrinome Clockwise|
|Q|Clear Board|

## Current Features

* Basic gameplay
  * Tetrinome translations
  * Tetrinome rotations
  * Line clearings
* Wall kicks
* Fancy Animations
  * Line clearings
  * Instant drops
* Shadow piece
* Instant drops

## Potential Future Features

For now I'm satisfied with what the I achieved considering it was a side project to try out game development and Rust, but here are features I would add if I return to the project:

* State management system
  * Pause state
  * Game Over state
* Next tetrinome queue
* Sounds & music
