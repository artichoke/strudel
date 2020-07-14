#!/usr/bin/env ruby
# frozen_string_literal: true

ITERATIONS = 10_000
KEYS = 1_000

indexes = KEYS.times.map { |_i| rand(KEYS) }

h = {}

# Setup a small map
8.times do |i|
  h[i] = i
end

# Query a small map with hits and misses
ITERATIONS.times do |i|
  h[i % 16]
end

h = {}

# Set the same key-value pair repeatedly
ITERATIONS.times do
  indexes.each do |i|
    h[i] = i
  end
end

h.clear

ITERATIONS.times do
  indexes.each do |i|
    h[i] = nil
  end
  indexes.each do |i|
    h[i] = i
  end
end

ITERATIONS.times do
  indexes.each do |i|
    raise unless h[i] == i
  end
end

h.each_pair do |key, value|
  raise unless key == value
end

h.reject! do |key, _value|
  next false if key == indexes[0]
  next false if key == indexes[-1]

  true
end

ITERATIONS.times do
  h.each_pair do |key, value|
    raise unless key == value
  end
end

ITERATIONS.times do
  h.each_pair do |key, _value|
    h.delete(key)
  end
end

raise unless h.empty?
