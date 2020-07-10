#!/usr/bin/env ruby

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
h.reject! do |key, value|
  reject_this = reject
  reject = !reject
  reject_this
end

h.each_pair do |key, value|
  raise unless key == value
  raise unless key % 2 == 1
end

h.each_pair do |key, value|
  h.delete(key)
end

raise unless h.empty?
