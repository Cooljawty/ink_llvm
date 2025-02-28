#pragma once
typedef struct {char* buffer; unsigned int size; } string;

typedef void* Story;
typedef struct{ char* text; char* tags; } Choice;

extern Story Step(Story);

///Returns a handel to a new instance of this story
extern Story NewStory();

///Steps through the given Story handel returning a line of content
extern string* Continue(Story);

///Steps through the given Story handel returning all lines of content until
///the story reaches a choice point/end of story
extern string* ContinueMaximally(Story);

///Returns false if story requires a choice selection or otherwise cannot continue
///it's control flow
extern bool CanContinue(Story);

///Returns the number of choices availabe from a given story handel
extern unsigned int ChoiceCount(Story);

///Returns a choice object at a given index from the given story handel
extern Choice GetChoice(Story, unsigned int);

///Selects the choice at a given index for the given story handel
///Note: Does not continue story
extern void ChooseChoiceIndex(Story, unsigned int);

/*TODO:
extern char* SaveToJson(Story)
extern void LoadJson(Story)

extern void ChoosePathString(car*)
extern void ObserveVariable(void*, func*)

List stuff:
...
*/

///C wrapper methods
string* new_string()
{
	string* new_string = malloc(sizeof(string));
	new_string->buffer = NULL;
	new_string->size = 0;
	return new_string;
}

void free_string(string* self)
{
	if(self == NULL) return;

	free(self->buffer);
	free(self);

	self = NULL;
}
//Write buf to string
//ref: https://doc.rust-lang.org/std/io/trait.Write.html#tymethod.write
unsigned int write_string(string* self, char* buf)
{
	unsigned int new_size = (self->size > 0 ? self->size : 1) + strlen(buf);
	self->size = new_size;

	self->buffer = realloc(self->buffer, self->size);
	strcat(self->buffer, buf);

	return self->size;
}
//Writes chars from the string to buf
//ref:https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read
unsigned int read_string(string* self, char* buf)
{
	buf = realloc(buf, self->size);
	strcpy(buf, self->buffer);

	return self->size;
}

//Ensures all characters in the string's buffer is sent to the strings destination
//ref: https://doc.rust-lang.org/std/io/trait.Write.html#tymethod.flush
void flush_string(string* self)
{
	//Debug: fush to stdout
	if(self->size > 0) printf("%s", self->buffer);

	self->buffer = realloc(self->buffer, 0);
	self->size = 0;
}
