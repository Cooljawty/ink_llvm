;Content Strings
@story.str_0 =					constant [ x i8] c"Hello!\00"
@story.str_1 =					constant [ x i8] c"\n\00"

@story.choice_0.str_0 =			constant [ x i8] c"Chose \00"
@story.choice_0.str_choice =	constant [ x i8] c"A\00"
@story.choice_0.str_1 =			constant [ x i8] c" the first\00"

@story.choice_1.str_0 =			constant [ x i8] c"Or \00"
@story.choice_1.str_choice =	constant [ x i8] c"B\00"
@story.choice_1.str_1 =			constant [ x i8] c" the second\00"

@story.gather_0.str_0 =			constant [ x i8] c"The end\00"

;IO Functions
declare i32 @puts(i8* nocapture) nounwind

%call_chain_type =			type { ptr, ptr } ; ptr up, ptr ret
%promise_type =				type { i32, ptr } ; { choice_index: i32, call_chain: {ptr, ptr}}

;Runtime Functions, takes handel or null
define ptr @Step(ptr story_handel) {
entry:
%new_instance =				icmp ne ptr story_handel, null
							br i1 %new_instance label %initilize, label %load_promise
initilize:
%new_instance_handel =		call ptr @__root()
							br label %load_promise

load_promise:
%handel =					phi ptr [%new_instance_handel, %initilize], [%story_handle, %entry]

%promise.addr =				call ptr @llvm.coro.promise(ptr %handel, i32 4, i1 false) ; TODO: Get target platform alignment
%promise =					load %promise_type, ptr %promise.addr

%up_handel.addr =			getelementptr %promise_type, %promise_type %promise, i32 0, i32 1, i32 0
%up_handel =				load ptr, ptr %up_handel.addr

%ret_handel.addr =			getelementptr %promise_type, %promise_type %promise, i32 0, i32 1, i32 1
%ret_handel =				load ptr, ptr %ret_handel.addr

%end_of_knot =				call i1 @llvm.coro.done(ptr %handel)
							br i1 %end_of_knot, label %done, label %continuing
done:
							call void @llvm.coro.destroy(%handel)
%diverting =				icmp ne ptr %up_handel, null
							br i1 %diverting, label %divert, label %done2
done2:
%return_from_tunnel =		icmp ne ptr %ret_handel, null
							br i1 %return_from_tunnel, label %call_ret, label %end
divert:
%has_return_handel =		icmp ne ptr %ret_handel, null
							br i1 %has_return_handel, label %delete_chain, label %call_up
delete_chain:
%parent_handel =			phi ptr [%ret_handel, %divert], [%ret_chain_handel, %delete_chain]
;Getting ret handel
%ret_promise.addr =			call ptr @llvm.coro.promise(ptr %parent_handel, i32 4, i1 false) ; TODO: Get target platform alignment
%ret_promise =				load %promise_type, ptr %promise.addr
%ret_chain_handel.addr =	getelementptr %promise_type, %promise_type %ret_promise, i32 0, i32 1, i32 1
%ret_chain_handel =			load ptr, ptr %ret_chain_handel.addr

							call void @llvm.coro.destroy(%parent_handel)
;Looping
%end_of_chain =				icmp eq ptr %ret_chain_handel, null
							br i1 %end_of_chain, label %call_up, label %delete_chain

continuing:
%tunneling =				icmp ne ptr %up_handel, null
							br i1 %tunneling, label %call_up, label %resume
call_ret:
							br label %resume
call_up:
							br label %resume
resume:
%resume_handel =			phi ptr [%handel, %continuing], [%up_handel, %call_up], [%ret_handel, %call_ret]
							call void @llvm.coro.resume(ptr %resume_handel)
							;TODO: call flush()
							ret ptr %resume_handel
end:
							ret ptr %handel
}


; Story
define ptr __root() presplitcoroutine {
entry:
%promise =					alloca %promise_type
%id =						call token @llvm.coro.id(i32 <align>, ptr %promise, ptr null, ptr null)
%alloc_flag =				call i1 @llvm.coro.alloc(token %id)
							br i1 %alloc_flag, label %frame_alloc, label %begin
frame_alloc:
%size =						call @llvm.coro.size.i32()
%alloc =					call ptr @malloc(i32 %size)
							br %begin
begin:
%phi_alloc =				phi ptr [null, %entry], [%frame_alloc, %alloc]
%handle =					call noalias ptr @llvm.coro.begin(token %id, ptr %phi_alloc)

suspend:
%unused =					call i1 @llvm.coro.end(null, i1 false, token none)
							ret ptr %handel
error:

destroy:
%frame_mem =				call ptr @llvm.coro.free(token %id, ptr %handle)
%free_frame =				icmp ne ptr %frame_mem, null
							br i1 %free_frame, label %frame_free, label %end
frame_free:
							call void @free(ptr %frame_mem)
							br label %end
end:

story:
							;"Hello!"
%str_.0 =					getelementptr [ x i8], ptr @str_0.0, i32 0, i32 0
							call i32 @puts(i8* %str_.0)

							;""
%str_.1 =					getelementptr [ x i8], ptr @str_0.1, i32 0, i32 0
							call i32 @puts(i8* %str_.1)

							br 0.choice_point_0

0.choice_point_0:
							; Choice 0: { text:i8* = @str_0.choice_0 .. @str_0.choice_0.choice}
							; Choice 1: { text:i8* = @str_0.choice_1 .. @str_0.choice_1.choice}

							%save_0.choice_point_0 = call token @llvm.coro.save(ptr %handel)
							%suspend_0.choice_point_0 = call i1 @llvm.coro.suspend(token %save_0.choice_point_0, i1 false)
							switch i8 %suspend_0.choice_point_0, %suspend, [i8 0, %resume_0.choice_point_0
																			i8 1, label %destroy]
resume_0.choice_point_0:
%choice_0_ptr =				%promise_type, %promise, i32 0 ;Load choice selection index
%choice_0_index =			load i32, ptr %choice_0_ptr
							switch i8 %choice_0_index %error, [i32 0, %0.choice_0 i32 1, %0.choice_1]

0.choice_0:					;"* Chose [A] the first"

							;"Chose "
%str_choice_0.0 =			getelementptr [ x i8], ptr @str_0.choice_0, i32 0, i32 0
							call i32 @puts(i8* %str_choice_0.0)

							;" the first"
%str_choice_0.1 =			getelementptr [ x i8], ptr @str_0.choice_0.1, i32 0, i32 0
							call i32 @puts(i8* %str_choice_0.1)

							br 0.gather_0

0.choice_1:					;"* Or [B] the second"

							;"Or "
%str_choice_1.0 =			getelementptr [ x i8], ptr @str_0.choice_1, i32 0, i32 0
							call i32 @puts(i8* %str_choice_1.0)

							;" the second"
%str_choice_1.1 =			getelementptr [ x i8], ptr @str_0.choice_1.1, i32 0, i32 0
							call i32 @puts(i8* %str_choice_1.1)

							br 0.gather_0

0.gather_0:					;"-"

							;"The end"
%0.gather_0.0 =				getelementptr [ x i8], ptr @str_0.choice_1.choice, i32 0, i32 0
							call i32 @puts(i8* %0.gather_0.0)
}
