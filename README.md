Quik Image Tagger
=================

This is a little command-line utility which will help you quickly classify images into categories.

It won't do any fancy sub-image segmentation or anything. Each image has one tag.

For example, let's say you have a directory `animals/` full of pictures of cats and fishes, and you want to tag each picture as either a "cat" picture or a "fish" picture. You can create a directory with subfolders `cat/` and `fish/`, and run the program like so:
```
$ ls dataset/
cat fish
$ cargo nightly build --release
$ ./target/debug/tagimg -i animals -o dataset
```
this will create a webserver on port 8000. Visit `http://localhost:8000/` and start clicking buttons - as you do, images will be moved from `animals/` into the appropriate folder under `dataset/`.
