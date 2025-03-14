== K1 ===
start of K1

end of K1's root
->END

	= K1_1
start of K1-1

end of K1-1

== function f1  =
f1() just prints this line
then returns {true}
~ return true
  == K2

= K2_1
start of k2-1

end of k2-1
	== function f2( x, b)
	f2({x},{b}):
	~ temp l = 2.34
	~ return x * l + b

===== function f3 (ref x) =====
f3() alters x: {x} by 1
~ x += 1
