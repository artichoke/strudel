#! /usr/bin/env ruby
# frozen_string_literal: true

puts "Ruby version: #{RUBY_VERSION}"

h = {}
1000.times do |i|
  h[i] = i.to_s
end

count = 50_000_000

indexes = count.times.map { rand(1000) }

h2 = {}

500_000.times { h.values }
500_000.times { h.keys }
indexes.each { |i| h[i] }
indexes.each { |i| h2[i] = i.to_s }
indexes.each { |i| h.delete(i) }
