== function Basic ==
No args here!
-> Done

== function One_arg (x) ==
{x}
-> Done

== function Many_args (x, y, z) ==
{x}, {y}, and {z}
-> Done

== function One_refrence (ref x) ==
x:= {x} + 1
~ x = x+1
-> Done

== function With_return_divert (-> ret) ==
Going back.. 
-> ret

== function Refrence_to_divert (ref ->ret) ==
Going back.. to HELL!!
~ ret = ->hell
-> ret
