#!/usr/bin/env ruby
# frozen_string_literal: true

ITERATIONS = 10
KEYS = 1_000_000

h = {}

KEYS.times do |i|
  h[i] = i
end

h.clear

KEYS.times do |i|
  h[i] = nil
end

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
