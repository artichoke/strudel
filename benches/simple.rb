#!/usr/bin/env ruby
# frozen_string_literal: true

ITERATIONS = 10_000
KEYS = 1_000

indexes = KEYS.times.map { |i| rand(KEYS) }

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

=begin
KEYS.times do |i|
  h[i] = i
end

KEYS.times do |i|
  raise unless h[i] == i
end

h.each_pair do |key, value|
  raise unless key == value
end

reject = true
h.reject! do |_key, _value|
  reject_this = reject
  reject = !reject
  reject_this
end

h.each_pair do |key, value|
  raise unless key == value
  raise unless key % 2 == 1 # rubocop:disable Style/EvenOdd
end

h.each_pair do |key, _value|
  h.delete(key)
end

raise unless h.empty?
=end
