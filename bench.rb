#! /usr/bin/env ruby

require 'benchmark'

puts "Ruby version: #{RUBY_VERSION}"

h = {}
1000.times do |i|
  h[i] = i.to_s
end

count = 50_000_000

indexes = count.times.map { rand(1000) }

h2 = {}
Benchmark.bm do |x| 
  x.report("values") { 500_000.times { h.values } }
  x.report("keys  ") { 500_000.times { h.keys } }
  x.report("find  ") { indexes.each { |i| h[i] } }
  x.report("insert") { indexes.each { |i| h2[i] = i.to_s } }
  x.report("delete") { indexes.each { |i| h.delete(i) } }
end