#pragma once
typedef void* Story;
typedef struct{ char* text; char* tags; } Choice;

extern Story Step(Story);

///Steps through the given Story handel returning a line of content
extern char* Continue(Story);

///Steps through the given Story handel returning all lines of content until
///the story reaches a choice point/end of story
extern char* ContinueMaximally(Story);

///Returns the number of choices availabe from a given story handel
extern unsigned int ChoiceCount(Story);

///Returns a choice object at a given index from the given story handel
extern Choice GetChoice(Story, unsigned int);

///Selects the choice at a given index for the given story handel
///Note: Does not continue story
extern Choice ChooseChoiceIndex(Story, unsigned int);

/*TODO:
extern char* SaveToJson(Story)
extern void LoadJson(Story)

extern void ChoosePathString(car*)
extern void ObserveVariable(void*, func*)

List stuff:
...
*/
