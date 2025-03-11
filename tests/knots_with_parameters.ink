== Basic ==
No args here!
-> Done

== One_arg (x) ==
{x}
-> Done

== Many_args (x, y, z) ==
{x}, {y}, and {z}
-> Done

== One_refrence (ref x) ==
x:= {x} + 1
~ x = x+1
-> Done

== With_return_divert (-> ret) ==
Going back.. 
-> ret

== Refrence_to_divert (ref ->ret) ==
Going back.. to HELL!!
~ ret = ->hell
-> ret
