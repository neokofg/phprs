<?php

function main(): void {
    // Try-catch basic
    try {
        echo "In try block";
    } catch (Exception $e) {
        echo "Caught exception";
    }

    // Try-catch-finally
    try {
        echo "Try block";
    } catch (Exception $e) {
        echo "Caught";
    } finally {
        echo "Finally always runs";
    }

    // Multiple exception types
    try {
        echo "Risky code";
    } catch (InvalidArgumentException|RuntimeException $e) {
        echo "Caught multiple types";
    }

    // Just finally (no catch)
    try {
        echo "Will cleanup";
    } finally {
        echo "Cleanup";
    }

    echo "Exceptions parsed!";
}
