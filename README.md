# Electronic Chess Board

Welcome to the Electronic Chess Board project repository. This project combines an electronic chess board with a chess engine, offering a more tactile way to train against an AI opponent.

## Overview

The project's core objective is to deliver an interactive chess experience, allowing users to engage in matches against an AI opponent.
This AI opponent employs a minimax algorithm for strategic decision-making, with a variable search depth of 3 to 6 based on time availability. 
The chess board integrates 64 hall effect sensors to precisely detect piece positions, while the AI's moves are visually indicated using 64 LEDs.

## Code

In the [Code](/Code) directory, you'll find the firmware that powers the project. This firmware includes the chess engine responsible for AI decision-making.
The project is programmed in embedded rust, and is designed to run on the custom STM32F103C8T6 chess board.

## CAD

The [CAD](/Cad) directory contains KiCad schematics for the custom chess board PCB. Additionally, you'll find laser cutting files and 3D design files for the casing of the electronic chess board.
Manufacturing outputs for the PCB can be found in the [Manufacture](/Cad/ChessBoardKiCad/Manufacture) directory.

# Manual

## Starting the game

Initially the board will prompt the player to start the game as either black or white. Press the button while the screen is displaying the team you want to play as.

## Setting up the board

After starting the game the screen will promt you to setup the board. Arrange the 32 chess pieces on the board in the starting position, ensuring that the piece colour you selected is closest to yourself. At any time while setting up the board you can hold down the button, this will light leds on the board in positions where pieces are missing, or pieces should be removed.
